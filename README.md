# NanoPulse - open-source IoT toolkit

NanoPulse is an open-source toolkit for building, connecting and integrating
low-power IoT devices. The NanoPulse protocol is PHY-layer agnostic, and
therefore can work with various chipsets and modulations (currently SX1302/3 for
the NanoPulse Gateways and SX126X for the NanoPulse Device SDK, support for the
CC1101 is planned).

**Note:** This project is work-in-progress, the documentation, protocol and
software are still in development.

## Structure

- `docs/` - documentation source
- `examples/` - code examples
  - `nanopulse-rp/` - Raspberry Pi Pico device example
- `nanopulse/` - library for reading and writing NanoPulse frames
- `nanopulse-device/` - library for implementing NanoPulse-based devices
- `nanopulse-gateway/` - NanoPulse Gateway binary
- `nanopulse-hal-sx1302/` - library linking against Semtech SX1302 HAL
- `nanopulse-server/` - NanoPulse Server binary
- `nanopulse-structs/` - library providing shared structs

## Documentation and binaries

Please refer to the [NanoPulse](https://www.nanopulse.dev) website for
documentation and pre-compiled binaries.

## Building from source

### Requirements

A [Nix](https://nixos.org/download.html) environment is provided, which will
automatically provide an environment with all development dependencies
available. For this, you need:

- [Nix](https://nixos.org/download.html)
- [Docker](https://docs.docker.com/get-started/) (for cross-compiling using
  `cross`)

Alternatively, you can install all development requirements manually. Please
refer to the packages listed in `shell.nix`.

To activate the Nix shell, execute at the root of this repository:

```bash
nix-shell
```

You also need to install `cross` (which will be installed to `.cargo/bin`,
relative to the root of this repository).

```bash
just install-dev-dependencies
```

### Available commands

To list all available commands, execute:

```bash
just -l
```

## License

NanoPulse is distributed under the Apache-2.0 license.
