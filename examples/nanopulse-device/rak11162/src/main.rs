#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, Ordering};
use heapless::index_map::FnvIndexMap;

// logging
use defmt::*;

use {defmt_rtt as _, panic_probe as _};

// embassy
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Delay, Duration, Timer};

// hal
use embassy_stm32::{
    bind_interrupts, dma,
    flash::{self, Flash},
    gpio::{Level, Output, Speed},
    mode, peripherals,
    rng::{self, Rng},
    spi::Spi,
    usart::{self, Uart},
};
use embedded_io_async::Write;

// nanopulse
use nanopulse::frames::{Buffer, Payload, configuration, telemetry};
use nanopulse_device::{
    device,
    lora_phy::{
        LoRa,
        sx126x::{self, Stm32wl, Sx126x, TcxoCtrlVoltage},
    },
    region,
};

// nanopulse
use nanopulse_device::context::{
    Counters, RootSecurity, get_counters, get_root_security, write_counters, write_root_security,
};

use self::iv::{InterruptHandler, Stm32wlInterfaceVariant, SubghzSpiDevice};

mod iv;

bind_interrupts!(struct Irqs{
    SUBGHZ_RADIO => InterruptHandler;
    USART1 => usart::InterruptHandler<peripherals::USART1>;
    RNG => rng::InterruptHandler<peripherals::RNG>;
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_CHANNEL4 => dma::InterruptHandler<peripherals::DMA1_CH4>;
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
});

// Channels for communication between tasks
static RX_CHANNEL: Channel<CriticalSectionRawMutex, Payload, 1> = Channel::new();
static TX_CHANNEL: Channel<CriticalSectionRawMutex, Payload, 1> = Channel::new();

type SpiType =
    SubghzSpiDevice<Spi<'static, embassy_stm32::mode::Async, embassy_stm32::spi::mode::Master>>;
type IvType = Stm32wlInterfaceVariant<Output<'static>>;
type LoRaType = LoRa<Sx126x<SpiType, IvType, Stm32wl>, Delay>;
type FlashType = Flash<'static, flash::Blocking>;
type DeviceType = device::Device<LoRaType, Rng<'static, peripherals::RNG>, FlashType, 8, 8, 1, 6>;

static TELEMETRY_INTERVAL: AtomicU32 = AtomicU32::new(30);

#[derive(Debug)]
pub enum EspError {
    Uart,
    Timeout,
    BufferFull,
    UnexpectedResponse,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("init");

    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.msi = Some(MSIRange::RANGE48M);
        config.rcc.sys = Sysclk::MSI;
        config.rcc.mux.rngsel = mux::Rngsel::MSI;
        config.enable_debug_during_sleep = true;
    }
    let p = embassy_stm32::init(config);
    let mut rng = Rng::new(p.RNG, Irqs);

    info!("setup esp");
    let esp_en = Output::new(p.PA0, Level::Low, Speed::Low);
    let esp_uart = Uart::new(
        p.USART1,
        p.PB7,
        p.PB6,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Irqs,
        Default::default(),
    )
    .unwrap();

    info!("setup LoRa radio");
    let tx_pin = Output::new(p.PC13, Level::Low, Speed::VeryHigh);
    let rx_pin = Output::new(p.PB8, Level::Low, Speed::VeryHigh);
    let spi = Spi::new_subghz(p.SUBGHZSPI, p.DMA1_CH1, p.DMA1_CH2, Irqs);
    let spi: SpiType = SubghzSpiDevice(spi);
    let use_high_power_pa = true;
    let config = sx126x::Config {
        chip: Stm32wl { use_high_power_pa },
        tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: true,
        rx_boost: false,
    };
    let iv: IvType =
        Stm32wlInterfaceVariant::new(Irqs, use_high_power_pa, Some(rx_pin), Some(tx_pin), None)
            .unwrap();
    let lora: LoRaType = LoRa::new(Sx126x::new(spi, iv, config), true, Delay)
        .await
        .unwrap();

    info!("reading context from flash memory");
    let mut flash: FlashType = Flash::new_blocking(p.FLASH);
    let mut root_security_ctx = get_root_security(&mut flash);
    let mut counters_ctx = get_counters(&mut flash);
    if root_security_ctx.is_empty() {
        warn!("root-security is empty, generating new key-pair + PIN");
        root_security_ctx = RootSecurity::new(&mut rng);
        write_root_security(&mut flash, &root_security_ctx);

        warn!("resetting counters");
        counters_ctx = Counters::default();
        write_counters(&mut flash, &counters_ctx);
    }

    info!("public key: {:02x}", root_security_ctx.public_key);
    trace!("secret key: {:02x}", root_security_ctx.secret_key);
    debug!("pin: {:02x}", root_security_ctx.pin);

    info!("setup NanoPulse device");
    let device: DeviceType = device::Device::new(
        lora,
        rng,
        flash,
        root_security_ctx,
        counters_ctx,
        &region::lora::eu868::REGION,
        device::DeviceConfig {
            data_rate: 5,
            vendor_id: [0x00, 0x00, 0x00, 0x01],
            profile_id: [0x00, 0x01],
            version_id: [0x00, 0x00],
        },
    );

    spawner.spawn(unwrap!(device_loop(
        device,
        RX_CHANNEL.sender(),
        TX_CHANNEL.receiver()
    )));

    spawner.spawn(unwrap!(downlink_handler_loop(
        TX_CHANNEL.sender(),
        RX_CHANNEL.receiver(),
    )));

    spawner.spawn(unwrap!(location_loop(
        esp_en,
        esp_uart,
        TX_CHANNEL.sender()
    )));
}

#[embassy_executor::task]
async fn device_loop(
    mut d: DeviceType,
    rx_sender: Sender<'static, CriticalSectionRawMutex, Payload, 1>,
    tx_receiver: Receiver<'static, CriticalSectionRawMutex, Payload, 1>,
) {
    info!("starting device_loop");
    loop {
        match d.next().await {
            Ok(Some(pl)) => {
                info!("received {} frame", pl.frame_type());
                rx_sender.send(pl.payload).await;
            }
            Ok(None) => {
                if let Ok(pl) = tx_receiver.try_receive() {
                    info!("sending {} frame", pl.frame_type());
                    if let Err(e) = d.send(pl).await {
                        error!("send error: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("device_loop error: {}", e);
            }
        }

        if d.has_rx_buffer() {
            continue;
        }

        tx_receiver.ready_to_receive().await;
    }
}

#[embassy_executor::task]
async fn downlink_handler_loop(
    tx_sender: Sender<'static, CriticalSectionRawMutex, Payload, 1>,
    rx_receiver: Receiver<'static, CriticalSectionRawMutex, Payload, 1>,
) {
    info!("starting downlink_handler loop");
    loop {
        let pl = rx_receiver.receive().await;
        match pl {
            Payload::Configuration(pl) => {
                if let configuration::Payload::Request(config_pl) = pl.payload {
                    if config_pl.len() != 4 {
                        warn!("invalid configuration payload length");
                        continue;
                    }

                    let interval = u32::from_le_bytes(config_pl[0..4].try_into().unwrap());
                    TELEMETRY_INTERVAL.store(interval, Ordering::Relaxed);

                    // acknowledge config request
                    tx_sender
                        .send(Payload::Configuration(configuration::Configuration {
                            is_resp: true,
                            payload: configuration::Payload::Response(
                                configuration::ResponsePayload {
                                    request_counter: pl.counter as u16,
                                    ..Default::default()
                                },
                            ),
                            ..Default::default()
                        }))
                        .await;
                }
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn location_loop(
    mut esp_en: Output<'static>,
    mut esp_uart: Uart<'static, mode::Async>,
    tx_sender: Sender<'static, CriticalSectionRawMutex, Payload, 1>,
) {
    info!("starting location loop");
    loop {
        info!("scanning for wifi aps");
        let mut b = Buffer::new();

        esp_en.set_high();
        let aps = wifi_scan(&mut esp_uart).await.unwrap();
        esp_en.set_low();

        for (k, v) in aps.iter() {
            b.extend_from_slice(k).unwrap();
            b.push((-1 * v) as u8).unwrap();
        }

        tx_sender
            .send(Payload::Telemetry(telemetry::Telemetry {
                payload: b,
                ..Default::default()
            }))
            .await;

        Timer::after(Duration::from_secs(
            TELEMETRY_INTERVAL.load(Ordering::Relaxed).into(),
        ))
        .await;
    }
}

async fn at_command(
    uart: &mut Uart<'static, mode::Async>,
    command: &str,
    response: &mut [u8],
) -> Result<usize, EspError> {
    uart.write_all(command.as_bytes())
        .await
        .map_err(|_| EspError::Uart)?;
    uart.write_all(b"\r\n").await.map_err(|_| EspError::Uart)?;

    let mut stored = 0;
    let mut chunk = [0u8; 2048];
    let mut tail = [0u8; 9];
    let mut tail_len = 0usize;

    loop {
        let n = uart
            .read_until_idle(&mut chunk)
            .await
            .map_err(|_| EspError::Uart)?;

        for &b in &chunk[..n] {
            if stored < response.len() {
                response[stored] = b;
                stored += 1;
            }

            if tail_len < tail.len() {
                tail[tail_len] = b;
                tail_len += 1;
            } else {
                tail.copy_within(1.., 0);
                tail[tail.len() - 1] = b;
            }

            let current = &tail[..tail_len];

            if current.ends_with(b"\r\nOK\r\n") || current.ends_with(b"\r\nready\r\n") {
                return Ok(stored);
            }

            if current.ends_with(b"\r\nERROR\r\n") {
                return Ok(stored);
            }
        }
    }
}

async fn wifi_scan(
    uart: &mut Uart<'static, mode::Async>,
) -> Result<FnvIndexMap<[u8; 6], i16, 8>, EspError> {
    let mut response = [0u8; 2048];
    at_command(uart, "AT", &mut response).await?;
    at_command(uart, "AT+CWMODE=1", &mut response).await?;
    at_command(uart, "AT+CWLAPOPT=,12", &mut response).await?;
    let n = at_command(uart, "AT+CWLAP", &mut response).await?;
    Ok(parse_cwlap(&response[..n]))
}

fn parse_cwlap(buffer: &[u8]) -> FnvIndexMap<[u8; 6], i16, 8> {
    let mut out = FnvIndexMap::<_, _, 8>::new();

    for line in buffer.split(|&b| b == b'\n') {
        let line = trim(line);

        if !line.starts_with(b"+CWLAP:(") {
            continue;
        }

        let Some(end) = line.iter().position(|&b| b == b')') else {
            continue;
        };

        let entry = &line[8..end];

        let Some(comma) = entry.iter().position(|&b| b == b',') else {
            continue;
        };

        // Parse RSSI
        let Ok(rssi_str) = core::str::from_utf8(&entry[..comma]) else {
            continue;
        };

        let Ok(rssi) = rssi_str.parse::<i16>() else {
            continue;
        };

        // Extract MAC between quotes
        let mac_part = &entry[comma + 1..];

        let Some(start) = mac_part.iter().position(|&b| b == b'"') else {
            continue;
        };

        let Some(end) = mac_part[start + 1..].iter().position(|&b| b == b'"') else {
            continue;
        };

        let mac_str = &mac_part[start + 1..start + 1 + end];

        // MAC must contain exactly 17 characters.
        if mac_str.len() != 17 {
            continue;
        }

        let mut mac = [0u8; 6];
        let mut valid = true;

        for i in 0..6 {
            let pos = i * 3;

            if i < 5 && mac_str[pos + 2] != b':' {
                valid = false;
                break;
            }

            let Ok(hex) = core::str::from_utf8(&mac_str[pos..pos + 2]) else {
                valid = false;
                break;
            };

            let Ok(value) = u8::from_str_radix(hex, 16) else {
                valid = false;
                break;
            };

            mac[i] = value;
        }

        if !valid {
            continue;
        }

        if out
            .get(&mac)
            .map(|v| rssi > *v)
            .unwrap_or_else(|| !out.is_full())
        {
            out.insert(mac, rssi).unwrap();
        }
    }

    out
}

fn trim(mut s: &[u8]) -> &[u8] {
    while s.first().is_some_and(|b| b.is_ascii_whitespace()) {
        s = &s[1..];
    }
    while s.last().is_some_and(|b| b.is_ascii_whitespace()) {
        s = &s[..s.len() - 1];
    }
    s
}
