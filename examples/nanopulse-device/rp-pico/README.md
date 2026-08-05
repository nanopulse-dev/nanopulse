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

**Note:** The `nanopulse_device` crate is compiled with the `no_flash_write`
flag enabled in this example. This disables flashing the security context to the
RP2040 flash. Therefore, each time when you turn on the device, a new key-pair
and PIN is generated! If you would like to persist the security-context, remove
this feature flag from `Cargo.toml` and recompile & flash the example.

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
