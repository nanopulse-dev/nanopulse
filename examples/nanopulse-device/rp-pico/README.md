# Raspberry Pi Pico example

This example demonstrates a device which sends temperature telemetry and which
has a state controllable LED. It uses the EU868 region.

## Requirements

To compile and flash this example, You need a working Rust environment and
`probe-rs` installed. You also need the following hardware:

- Raspberry Pi Pico
- Raspberry Pi Debug Probe
- Waveshare SX1262 LoRa Node Module for Raspberry Pi Pico

## Running the example

**Note:** On the first usage, this will create a key-pair + PIN, which will be
persistent in the flash memory of the chip. Reflashing will not overwrite this
key-pair. To erase the key-pair and PIN, execute the command
`probe-rs erase --chip RP2040` (make sure 2MB of memory is erased).
Alternatively, you can use the `no_flash_write` feature of the
`nanopulse_device` crate, which will disable writing to flash.

To compile, flash and run the example and view its log output, execute:

```bash
debug cargo run --release
```

Please look for `pin: ...` in the logs. E.g.:

```
0.202602 [DEBUG] pin: [ca, 92, b6, 24] (example_nanopulse_rpi nanopulse-rp/src/main.rs:114)
```

This means you need to enter `ca92b624` as PIN in the NanoPulse Server to
complete the Key Exchange.
