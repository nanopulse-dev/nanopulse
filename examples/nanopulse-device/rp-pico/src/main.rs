#![no_std]
#![no_main]

// logging
use defmt::*;
use {defmt_rtt as _, panic_probe as _};

// embassy
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Delay, Duration, Timer};

// hal
use embassy_rp::{
    adc::{self, Adc, Channel as AdcChannel, Config as AdcConfig, InterruptHandler},
    bind_interrupts,
    clocks::RoscRng,
    dma,
    flash::{self, Flash},
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, FLASH},
    spi::{Config, Spi},
};
use nanopulse_device::embedded_hal_bus::spi::ExclusiveDevice;
use nanopulse_device::lora_phy::{
    LoRa,
    iv::GenericSx126xInterfaceVariant,
    sx126x::{self, Sx126x, Sx1262, TcxoCtrlVoltage},
};

// nanopulse
use nanopulse::frames::{Buffer, Payload, state, telemetry};
use nanopulse_device::context::{
    Counters, RootSecurity, get_counters, get_root_security, write_counters, write_root_security,
};
use nanopulse_device::{device, region};

// Interrupts
bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>;
    ADC_IRQ_FIFO => InterruptHandler;
});

// Channels for communication between tasks
static RX_CHANNEL: Channel<CriticalSectionRawMutex, Payload, 1> = Channel::new();
static TX_CHANNEL: Channel<CriticalSectionRawMutex, Payload, 1> = Channel::new();

// We need to define the exact types, as embassy_executor::task does not accept generics.
type SpiType = ExclusiveDevice<
    Spi<'static, embassy_rp::peripherals::SPI1, embassy_rp::spi::Async>,
    Output<'static>,
    Delay,
>;
type IvType = GenericSx126xInterfaceVariant<Output<'static>, Input<'static>>;
type LoRaType = LoRa<Sx126x<SpiType, IvType, Sx1262>, Delay>;
type FlashType = Flash<'static, FLASH, flash::Async, 2097152>;
type DeviceType = device::Device<LoRaType, RoscRng, FlashType, 8, 8, 1, 6>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("init");
    let p = embassy_rp::init(Default::default());
    let adc = Adc::new(p.ADC, Irqs, AdcConfig::default());

    info!("setup temp sensor");
    let temp_sensor = AdcChannel::new_temp_sensor(p.ADC_TEMP_SENSOR);

    info!("setup LoRa radio");
    let nss = Output::new(p.PIN_3, Level::High);
    let reset = Output::new(p.PIN_15, Level::High);
    let dio1 = Input::new(p.PIN_20, Pull::None);
    let busy = Input::new(p.PIN_2, Pull::None);
    let spi = Spi::new(
        p.SPI1,
        p.PIN_10,
        p.PIN_11,
        p.PIN_12,
        p.DMA_CH0,
        p.DMA_CH1,
        Irqs,
        Config::default(),
    );
    let spi: SpiType = ExclusiveDevice::new(spi, nss, Delay).unwrap();
    let config = sx126x::Config {
        chip: Sx1262,
        tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: true,
        rx_boost: false,
    };
    let iv: IvType = GenericSx126xInterfaceVariant::new(reset, dio1, busy, None, None).unwrap();
    let lora: LoRaType = LoRa::new(Sx126x::new(spi, iv, config), true, Delay)
        .await
        .unwrap();

    info!("setup LED pin");
    let led = Output::new(p.PIN_25, Level::Low);

    info!("reading context from flash memory");
    let mut flash: FlashType = Flash::new(p.FLASH, p.DMA_CH2, Irqs);
    let mut root_security_ctx = get_root_security(&mut flash);
    let mut counters_ctx = get_counters(&mut flash);
    if root_security_ctx.is_empty() {
        warn!("root-security is empty, generating new key-pair + PIN");
        root_security_ctx = RootSecurity::new(&mut RoscRng);
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
        RoscRng,
        flash,
        root_security_ctx,
        counters_ctx,
        &region::lora::eu868::REGION,
        device::DeviceConfig {
            data_rate: 5,
            vendor_id: [0x00, 0x00, 0x00, 0x00],
            profile_id: [0x00, 0x00],
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
        led
    )));

    spawner.spawn(unwrap!(temperature_loop(
        TX_CHANNEL.sender(),
        adc,
        temp_sensor
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
                if matches!(pl.payload, Payload::State(_)) {
                    info!("received state payload");
                    rx_sender.send(pl.payload).await;
                }
            }
            Ok(None) => {
                if let Ok(pl) = tx_receiver.try_receive() {
                    match pl {
                        Payload::Telemetry(pl) => {
                            info!("sending telemetry payload");
                            if let Err(e) = d.send_telemetry(pl).await {
                                error!("send telemetry error: {}", e);
                            }
                        }
                        Payload::State(pl) => {
                            info!("sending state payload");
                            if let Err(e) = d.send_state(pl).await {
                                error!("send state error: {}", e);
                            }
                        }
                        _ => {}
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
    mut led: Output<'static>,
) {
    info!("starting downlink_handler loop");
    loop {
        let pl = rx_receiver.receive().await;
        if let Payload::State(pl) = pl {
            if let state::Payload::Request(state_pl) = pl.payload {
                if state_pl[0] == 0x00 {
                    led.set_low();
                } else if state_pl[0] == 0x01 {
                    led.set_high();
                }

                // acknowledge state request
                tx_sender
                    .send(Payload::State(state::State {
                        payload: state::Payload::Response(state::ResponsePayload {
                            request_counter: pl.counter as u16,
                            ..Default::default()
                        }),
                        ..Default::default()
                    }))
                    .await;
            }
        }
    }
}

#[embassy_executor::task]
async fn temperature_loop(
    tx_sender: Sender<'static, CriticalSectionRawMutex, Payload, 1>,
    mut adc: Adc<'static, adc::Async>,
    mut temp_sensor: AdcChannel<'static>,
) {
    info!("starting temperature_loop");
    loop {
        info!("measuring temperature");
        let mut b = Buffer::new();
        let temp = adc.read(&mut temp_sensor).await.unwrap();
        let temp = convert_to_celsius(temp);
        b.extend_from_slice(&temp.to_le_bytes()).unwrap();
        tx_sender
            .send(Payload::Telemetry(telemetry::Telemetry {
                payload: b,
                ..Default::default()
            }))
            .await;

        Timer::after(Duration::from_secs(30)).await;
    }
}

fn convert_to_celsius(raw_temp: u16) -> i16 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    ((temp * 10.0) + 0.5 * sign) as i16
}
