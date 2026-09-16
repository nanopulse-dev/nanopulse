# Home Assistant

NanoPulse server can be installed as an Home Assistant app and can integrate
with the Home Assistant MQTT integration for automatic device discovery and
telemetry reporting. This guide explains how to setup NanoPulse within Home
Assistant and use a Raspberry Pi Pico, together with a Waveshare SX1262 module.
The Raspberry Pi Pico will periodically report the temperature (through its
built-in temperature sensor) and through Home Assistant the on-board LED can be
controlled.

> [!NOTE]
> There is also a
> [Raspberry Pi Pico 2W example](https://github.com/nanopulse-dev/nanopulse/tree/master/examples/nanopulse-device/rp-pico2w)
> which reports temperature and location through Wi-Fi access-point scanning
> (using [beaconDB](https://beacondb.net/) or
> [Google Geolocation API](https://developers.google.com/maps/documentation/geolocation/overview)).

## Hardware

### Home Assistant

In order to run apps within Home Assistant, you need a Home Assistant Operating
System (HA OS) setup (e.g. on a Raspberry Pi). Please refer to the
[Home Assistant installation instructions](https://www.home-assistant.io/installation/).

### Concentrator module

This guide uses an EU868 RAK2287 LoRa concentrator module (USB) + mPCIe to USB
board:

- [EU868 RAK2287 USB](https://store.rakwireless.com/products/wislink-concentrator-module-sx1302-rak2287-lorawan?variant=44510641619142)
- [mPCIe to USB Board](https://store.rakwireless.com/products/mpcie-to-usb-board)

This module must be plugged in one of the USB ports of the device running Home
Assistant (e.g. a Raspberry Pi).

### Device

This guide uses a Raspberry Pico module (with headers!) and a Raspberry Pi Debug
Probe, together with a Waveshare SX1262 module.

- [Raspberry Pi Pico](https://www.raspberrypi.com/products/raspberry-pi-pico)
- [Raspberry Pi Debug Probe](https://www.raspberrypi.com/products/debug-probe/)
- [Waveshare SX1262 module](https://www.waveshare.com/pico-lora-sx1262-868m.htm)

## Installation

### Mosquitto

Both the NanoPulse Gateway and NanoPulse Server require a MQTT broker and the
Home Assistant MQTT integration. To install the Mosquitto broker:

- Click _Settings_ > _Apps_
- Click _Install app_
- Click _Mosquitto broker_
- Click _install_

After installation:

- Click _Start_

### MQTT integration

To install the MQTT integration:

- Click _Settings_ > _Devices & services_
- Click _+ Add integration_
- Click _MQTT_
- From the list of MQTT integrations, click _MQTT_
- When prompted how to connect, click _Use the official Mosquitto Mqtt Broker
  app._

### NanoPulse repository

To add the NanoPulse apps repository:

- Click _Settings_ > _Apps_
- Click _Install app_
- Click the right-top menu (three dots), click _Repositories_
- Click _Add_
- Enter `https://github.com/nanopulse-dev/hassio-nanopulse` and click _Add_

### NanoPulse Gateway

To install the NanoPulse Gateway service:

- Click _Settings_ > _Apps_
- Click _Install app_
- Click the _NanoPulse Gateway_ app
- Click _Install_
- Click _Start_
- Click the _Log_ tab, to confirm it has started without errors

### NanoPulse Server

To install the NanoPulse Server:

- Click _Settings_ > _Apps_
- Click _Install app_
- Click the _NanoPulse Server_ app
- Click _Install_
- Click _Start_
- Enable _Show in sidebar_

You should see a _NanoPulse_ entry in the left menu of Home Assistant.

## Add gateway to NanoPulse

To add the gateway (which has the default name `np-gateway`) to NanoPulse:

- Click _NanoPulse_
- Within the NanoPulse UI
  - Click _Add workspace_
  - Create a workspace name `nanopulse`
  - Click _Gateways_
  - Click _Add gateway_
  - Create a gateway with name `np-gateway` and Region configuration `EU868`

## Prepare the device

### Raspberry Pi Pico

Please prepare the Raspberry Pi Pico device by connecting the Waveshare SX1262
module, Rapsbery Pi Debug Probe and USB cables. The device will be connected by
two USB cables to your computer:

1. Connecting to the Debug Probe (for flashing and debug output)
2. Providing power to the Raspberry Pi Pico

### Example code

The device code that will be used for this example can be found here:
[https://github.com/nanopulse-dev/nanopulse/tree/master/examples/nanopulse-device/rp-pico](https://github.com/nanopulse-dev/nanopulse/tree/master/examples/nanopulse-device/rp-pico).

In order to compile and flash this, you need a local copy of this repository:

```bash
git clone https://github.com/nanopulse-dev/nanopulse.git
```

### Software requirements

The NanoPulse Device stack is written in [Rust](https://rust-lang.org/) and uses
[probe-rs](https://probe.rs/) for flashing and debugging. You can use the
provided [Nix](https://nixos.org/download/) shell, or install Rust and probe-rs
manually.

#### Nix shell

At the root of the nanopulse repository, execute:

```bash
nix-shell
```

### Flash application

To flash the application to the Raspberry Pi Pico, execute the following
command:

```bash
cd examples/nanopulse-device/rp-pico
DEFMT_LOG=debug cargo run --release
```

Please take note of the following log (the pin value will be different):

```
0.292752 [DEBUG] pin: [ca, 55, aa, 55] (rp_pico rp-pico/src/main.rs:116)
```

**Note:** this means that the PIN of this device is `ca55aa55`. This will be
used in the next steps.

Leave the device on, it will periodically try to perform a key-exchange with the
server.

## Add device to NanoPulse

In the Home Assistant web-interface:

- Click _NanoPulse_
- Click _Gateways_
- Select the `np-gateway` and click _1 selected gateway(s)_ > _Allow
  key-exchange_

Within a minute, you should get a _New device_ popup.

- Enter the _PIN_, name it `test-device` and click _Accept_

Within a minute, you should see in the console logs (of the `cargo run ...`
command) that the key-exchange completes (`key-exchange OK`), that it activates
(`activation OK`) and that it starts sending telemetry.

## Device telemetry and state

The device provides temperature telemetry and exposes a LED state, which can be
controlled through Home Assistant. To view the device:

- Click _Settings_ > _Devices & services_
- Under _Configured_, click _MQTT_
- You should see a _test-device_ entry, click it

Under _Controls_, you should see _LED_ with a toggle to turn the LED on or off.
Please note that since the device sends data only every 30 seconds, it might
take up to this interval before the LED on the Raspberry Pi Pico responds.

Under _Sensors_, you should see a _Temperature_ sensor with the current
temperature.

Now you can start using this data with other Home Assistant devices. For
example, using a Zigbee switch, you can toggle the Raspberry Pi Pico LED and
using the Raspberry Pi Pico temperature sensor, you can control your
airconditioner or heater.
