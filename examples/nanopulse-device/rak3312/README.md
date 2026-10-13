# WisBlock RAK3312 example

This example demonstrates a WisBlock RAK3312 Wi-Fi tracker.

## Requirements

To compile and flash this example, You need a working Rust environment and
`probe-rs` installed. You also need to install the ESP toolchain using
[`espup`](https://github.com/esp-rs/espup).

## Hardware

This example uses the
[RAK3312](https://store.rakwireless.com/products/wisblock-core-module-rak3312-lora-wifi-ble)
module.

## Running the example

**Note:** On the first usage, this will create a key-pair + PIN, which will be
persistent in the flash memory of the chip. Reflashing will not overwrite this
key-pair. To erase the key-pair and PIN, execute the command
`probe-rs erase --chip esp32s3`. Alternatively, you can use the `no_flash_write`
feature of the `nanopulse_device` crate, which will disable writing to flash.

To compile, flash and run the example and view its log output, execute:

```bash
debug cargo run --release
```

Please look for `pin: ...` in the logs. E.g.:

```
0.483171 [DEBUG] pin: [39, 2f, bc, dd] (rak3312 rak3312/src/main.rs:117)
```

This means you need to enter `392fbcdd` as PIN in the NanoPulse Server to
complete the Key Exchange.
