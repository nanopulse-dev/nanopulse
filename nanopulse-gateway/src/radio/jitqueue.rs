use std::time::Duration;

use anyhow::Result;
use tracing::{debug, error, info, warn};

use nanopulse_structs::gateway::TxAckStatus;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TxMode {
    Immediate,
    Timestamped,
}

pub struct Item<T> {
    count: Duration,
    pre_delay: Duration,
    post_delay: Duration,
    packet: T,
}

pub trait TxPacket {
    fn get_id(&self) -> u32;
    fn get_time_on_air(&self) -> Result<Duration>;
    fn get_tx_mode(&self) -> TxMode;
    fn set_tx_mode(&mut self, tx_mode: TxMode);
    fn get_count(&self) -> Duration;
    fn set_count(&mut self, count: Duration);
}

pub struct Queue<T> {
    config: Config,
    items: Vec<Item<T>>,

    // This value holds the linear counter value after finishing the last downlink transmission. We
    // need to store this as once the downlink is scheduled, it is popped from the queue and we no
    // longer know until when the concentrator is busy transmitting.
    tx_count_finished: Duration,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Queue {
            config: Config::default(),
            items: vec![],
            tx_count_finished: Duration::default(),
        }
    }
}

pub struct Config {
    tx_start_delay: Duration,
    tx_margin_delay: Duration,
    tx_jit_delay: Duration,
    tx_max_advance_delay: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            tx_start_delay: Duration::from_micros(1500),
            tx_margin_delay: Duration::from_micros(1000),
            tx_jit_delay: Duration::from_micros(40000),
            tx_max_advance_delay: Duration::from_secs((3 + 1) * 128),
        }
    }
}

impl<T: TxPacket> Queue<T> {
    pub fn new(capacity: usize, config: Config) -> Queue<T> {
        info!(capacity = capacity, "Initializing JIT queue");

        Queue {
            config,
            items: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    pub fn full(&self) -> bool {
        self.items.len() == self.items.capacity()
    }

    pub fn pop(&mut self, count: Duration) -> Option<T> {
        match self.items.first() {
            None => return None,
            Some(v) => {
                // this might happen under high load
                if v.count < count {
                    error!(
                        pkt_count = ?v.packet.get_count(),
                        current_count = ?count,
                        "Dropping packet, packet is too old"
                    );
                    self.items.remove(0);
                    return None;
                }

                // too far in advance
                if v.count - count > v.pre_delay {
                    return None;
                }
            }
        }

        let item = self.items.remove(0);

        self.tx_count_finished = item.count + item.post_delay;
        Some(item.packet)
    }

    pub fn enqueue(&mut self, count: Duration, packet: T) -> Result<(), TxAckStatus> {
        match packet.get_tx_mode() {
            TxMode::Immediate => {
                info!(
                    pkt_id = packet.get_id(),
                    current_count = ?count,
                    "Enqueueing immediate packet"
                );
            }
            TxMode::Timestamped => {
                info!(
                    pkt_id = packet.get_id(),
                    pkt_count = ?packet.get_count(),
                    current_count = ?count,
                    "Enqueueing timestamped packet"
                );
            }
        }

        if self.full() {
            return Err(TxAckStatus::QueueFull);
        }

        let time_on_air = packet.get_time_on_air().map_err(|e| {
            error!(error = %e, "Get packet time on air error");
            TxAckStatus::InternalError
        })?;

        let mut item = Item {
            count: packet.get_count(),
            pre_delay: self.config.tx_start_delay + self.config.tx_jit_delay,
            post_delay: time_on_air,
            packet,
        };

        if item.packet.get_tx_mode() == TxMode::Immediate {
            item.packet.set_tx_mode(TxMode::Timestamped);

            // now + margin
            let mut asap_count = count + (2 * self.config.tx_jit_delay);

            // eventual collission with current tx not in queue, but already enqueued to radio
            let not_before_count =
                self.tx_count_finished + self.config.tx_margin_delay + item.pre_delay;
            if asap_count < not_before_count {
                asap_count = not_before_count;
            }

            // check for collisions
            if self.collision_test(count, item.pre_delay, item.post_delay) {
                for p2 in self.items.iter() {
                    asap_count =
                        p2.count + p2.post_delay + item.pre_delay + self.config.tx_margin_delay;
                    if !self.collision_test(asap_count, item.pre_delay, item.post_delay) {
                        break;
                    }
                }
            }

            item.count = asap_count;
            item.packet.set_count(asap_count);
        } else if item.packet.get_tx_mode() == TxMode::Timestamped
            && self.collision_test(item.count, item.pre_delay, item.post_delay)
        {
            return Err(TxAckStatus::CollisionPacket);
        }

        // is it too late to send this packet?
        if item.count < count
            || item.count - count
                < self.config.tx_start_delay
                    + self.config.tx_margin_delay
                    + self.config.tx_jit_delay
        {
            warn!(
                pkt_id = item.packet.get_id(),
                pkt_count = ?item.packet.get_count(),
                current_count = ?count,
                "Too late to enqueue packet"
            );

            return Err(TxAckStatus::TooLate);
        }

        // is it too early?
        if item.count - count > self.config.tx_max_advance_delay {
            warn!(
                pkt_id = item.packet.get_id(),
                pkt_count = ?item.packet.get_count(),
                current_count = ?count,
                "Too early to enqueue packet"
            );

            return Err(TxAckStatus::TooLate);
        }

        debug!(
            pkt_id = item.packet.get_id(),
            pkt_count = ?item.packet.get_count(),
            "Packet enqueued"
        );

        self.items.push(item);
        self.sort();

        Ok(())
    }

    fn sort(&mut self) {
        self.items.sort_by_key(|a| a.count)
    }

    fn collision_test(&self, count: Duration, pre_delay: Duration, post_delay: Duration) -> bool {
        if count < self.tx_count_finished + pre_delay + self.config.tx_margin_delay {
            // a packet is currently being txed
            return true;
        }

        for p2 in self.items.iter() {
            if count > p2.count {
                if count - p2.count <= pre_delay + p2.post_delay + self.config.tx_margin_delay {
                    return true;
                }
            } else if p2.count - count <= p2.pre_delay + post_delay + self.config.tx_margin_delay {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct TxPacketMock {
        time_on_air: Duration,
        tx_mode: TxMode,
        count: Duration,
    }

    impl TxPacket for TxPacketMock {
        fn get_time_on_air(&self) -> Result<Duration> {
            Ok(self.time_on_air)
        }

        fn get_tx_mode(&self) -> TxMode {
            self.tx_mode
        }

        fn get_id(&self) -> u32 {
            0
        }

        fn set_tx_mode(&mut self, tx_mode: TxMode) {
            self.tx_mode = tx_mode;
        }

        fn get_count(&self) -> Duration {
            self.count
        }

        fn set_count(&mut self, count: Duration) {
            self.count = count
        }
    }

    #[test]
    fn test_size() {
        let q: Queue<TxPacketMock> = Queue::new(10, Config::default());
        assert_eq!(10, q.items.capacity());
    }

    #[test]
    fn test_enqueue_full() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, Config::default());

        q.enqueue(
            Duration::from_nanos(100),
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count: Duration::ZERO,
            },
        )
        .unwrap();

        q.enqueue(
            Duration::from_nanos(100),
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count: Duration::ZERO,
            },
        )
        .unwrap();

        assert!(
            q.enqueue(
                Duration::from_nanos(100),
                TxPacketMock {
                    time_on_air: Duration::from_millis(100),
                    tx_mode: TxMode::Immediate,
                    count: Duration::ZERO,
                },
            )
            .is_err(),
            "jit queue should be full"
        );
    }

    #[test]
    fn test_enqueue_immediate() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, Config::default());
        let count = Duration::from_nanos(100);

        q.enqueue(
            count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count: Duration::ZERO,
            },
        )
        .unwrap();

        q.enqueue(
            count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Immediate,
                count: Duration::ZERO,
            },
        )
        .unwrap();

        let item = &q.items[0];
        assert_eq!(TxMode::Timestamped, item.packet.get_tx_mode());
        assert_eq!(Duration::from_micros(1500 + 40000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(count + Duration::from_millis(80), item.packet.get_count());

        let first_end_count = item.packet.get_count() + item.post_delay;

        let item = &q.items[1];
        assert_eq!(Duration::from_micros(1500 + 40000), item.pre_delay);
        assert_eq!(Duration::from_millis(100), item.post_delay);
        assert_eq!(
            first_end_count + item.pre_delay + q.config.tx_margin_delay,
            item.packet.get_count()
        );
    }

    #[test]
    fn test_pop_empty() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, Config::default());
        let item = q.pop(Duration::from_secs(1));
        assert!(item.is_none());
    }

    #[test]
    fn test_pop() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, Config::default());
        let count = Duration::from_secs(1);
        q.enqueue(
            count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count: Duration::from_secs(2),
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(2));
        assert!(item.is_some());
    }

    #[test]
    fn test_pop_too_far_in_future() {
        let mut q: Queue<TxPacketMock> = Queue::new(2, Config::default());
        let count = Duration::from_secs(1);
        q.enqueue(
            count,
            TxPacketMock {
                time_on_air: Duration::from_millis(100),
                tx_mode: TxMode::Timestamped,
                count: Duration::from_secs(2),
            },
        )
        .unwrap();

        let item = q.pop(Duration::from_secs(1));
        assert!(item.is_none());
    }
}
