#![no_std]
#![no_main]

use heapless::index_map::FnvIndexMap;

// logging
use defmt::*;
use defmt_rtt as _;
use esp_backtrace as _;

// embassy
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Delay, Duration, Timer};

// hal
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::rng::Rng as EspRng;
use esp_hal::spi::{
    Mode,
    master::{Config, Spi},
};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_storage::FlashStorage;

use nanopulse_device::embedded_hal_bus::spi::ExclusiveDevice;
use nanopulse_device::lora_phy::{
    LoRa,
    iv::GenericSx126xInterfaceVariant,
    sx126x::{self, Sx126x, Sx1262, TcxoCtrlVoltage},
};

// nanopulse
use nanopulse::frames::{Buffer, FrameType};
use nanopulse_device::context::{
    Counters, RootSecurity, get_counters, get_root_security, write_counters, write_root_security,
};
use nanopulse_device::device;
use nanopulse_device::region;

esp_bootloader_esp_idf::esp_app_desc!();

// Channels for communication between tasks
static RX_CHANNEL: Channel<CriticalSectionRawMutex, (FrameType, Buffer), 1> = Channel::new();
static TX_CHANNEL: Channel<CriticalSectionRawMutex, (FrameType, Buffer), 1> = Channel::new();

// We need to define the exact types, as embassy_executor::task does not accept generics.
type SpiType = ExclusiveDevice<Spi<'static, esp_hal::Async>, Output<'static>, Delay>;
type IvType = GenericSx126xInterfaceVariant<Output<'static>, Input<'static>>;
type LoRaType = LoRa<Sx126x<SpiType, IvType, Sx1262>, Delay>;
type FlashType = FlashStorage<'static>;
type DeviceType = device::Device<LoRaType, EspRng, FlashType, 8, 8, 1, 6>;

#[esp_hal::main]
async fn main(spawner: Spawner) {
    info!("init");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    let mut rng = EspRng::new();

    info!("setup LoRa radio");
    let nss = Output::new(peripherals.GPIO7, Level::High, OutputConfig::default());
    let sclk = peripherals.GPIO5;
    let mosi = peripherals.GPIO6;
    let miso = peripherals.GPIO3;
    let reset = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    let busy = Input::new(peripherals.GPIO48, InputConfig::default());
    let dio1 = Input::new(peripherals.GPIO47, InputConfig::default());
    let spi = Spi::new(
        peripherals.SPI2,
        Config::default()
            .with_frequency(Rate::from_khz(100))
            .with_mode(Mode::_0),
    )
    .unwrap()
    .with_sck(sclk)
    .with_mosi(mosi)
    .with_miso(miso)
    .into_async();
    let spi: SpiType = ExclusiveDevice::new(spi, nss, Delay).unwrap();
    let config = sx126x::Config {
        chip: Sx1262,
        tcxo_ctrl: Some(TcxoCtrlVoltage::Ctrl1V7),
        use_dcdc: false,
        rx_boost: true,
    };
    let iv: IvType = GenericSx126xInterfaceVariant::new(reset, dio1, busy, None, None).unwrap();
    let lora: LoRaType = LoRa::new(Sx126x::new(spi, iv, config), true, Delay)
        .await
        .unwrap();

    info!("reading context from flash memory");
    let mut flash = FlashStorage::new(peripherals.FLASH).multicore_auto_park();
    defmt::info!(
        "capacity = {} bytes (0x{:X})",
        flash.capacity(),
        flash.capacity()
    );
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
            profile_id: [0x00, 0x00],
            version_id: [0x00, 0x00],
        },
    );

    info!("setup wifi controller");
    // Use 64kB in the same region stack uses (dram_seg), for the heap.
    esp_alloc::heap_allocator!(size: 64000);
    // Use 64kB for the heap in the memory region reclaimed from the bootloader, which is otherwise unused.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);
    let wifi_controller =
        esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default()).unwrap();

    spawner.spawn(unwrap!(device_loop(
        device,
        RX_CHANNEL.sender(),
        TX_CHANNEL.receiver()
    )));

    spawner.spawn(unwrap!(telemetry_loop(
        TX_CHANNEL.sender(),
        wifi_controller
    )));
}

#[embassy_executor::task]
async fn device_loop(
    mut d: DeviceType,
    _rx_sender: Sender<'static, CriticalSectionRawMutex, (FrameType, Buffer), 1>,
    tx_receiver: Receiver<'static, CriticalSectionRawMutex, (FrameType, Buffer), 1>,
) {
    info!("starting device_loop");
    loop {
        match d.next().await {
            Ok(Some(_pl)) => info!("received payload"),
            Ok(None) => {
                if let Ok((frame_type, pl)) = tx_receiver.try_receive() {
                    match frame_type {
                        FrameType::Telemetry => {
                            info!("sending telemetry payload");
                            if let Err(e) = d.send_telemetry(&pl).await {
                                error!("send telemetry error: {}", e);
                            }
                        }
                        FrameType::State => {
                            info!("sending state payload");
                            if let Err(e) = d.send_state(&pl).await {
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
async fn telemetry_loop(
    tx_sender: Sender<'static, CriticalSectionRawMutex, (FrameType, Buffer), 1>,
    mut wifi_controller: esp_radio::wifi::WifiController<'static>,
) {
    info!("starting telemetry loop");
    loop {
        info!("scanning wifi");
        let mut b = Buffer::new();

        let mut wifi_result = FnvIndexMap::<_, _, 8>::new();
        let aps = wifi_controller
            .scan_async(&esp_radio::wifi::scan::ScanConfig::default().with_max(10))
            .await
            .unwrap();
        for ap in aps {
            if wifi_result
                .get(&ap.bssid)
                .map(|v| ap.signal_strength > *v)
                .unwrap_or_else(|| !wifi_result.is_full())
            {
                wifi_result.insert(ap.bssid, ap.signal_strength).unwrap();
            }
        }

        for (k, v) in wifi_result.iter() {
            b.extend_from_slice(k).unwrap();
            b.push((-1 * v) as u8).unwrap();
        }

        tx_sender.send((FrameType::Telemetry, b)).await;
        Timer::after(Duration::from_secs(30)).await;
    }
}
