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

#### Mise

[Mise](https://mise.jdx.dev/) is used for managing the development environment
dependencies. It handles both installing packages using your distro's
package-manager (currently `apt` and `pacman`) and installing per-project
dependencies (downloaded to `~/.local/share/mise` by default).

- [Installing Mise](https://mise.jdx.dev/installing-mise.html)
- [Shell setup](https://mise.jdx.dev/shell-setup.html)

#### Docker

Docker is used for cross-compiling binaries.

- https://docs.docker.com/get-started/

Alternatively, you can install all development requirements manually. Please
refer to the packages listed in `shell.nix`.

### Install dependencies

Please refer to the `mise.toml` file for dependencies that will be installed.
Alternatively, you can install these dependencies yourself.

```
# Install both [bootstrap.packages] and [tools]
mise bootstrap

# Install only [tools]
mise install
```

### Available commands

To list all available commands, execute:

```bash
just -l
```

## License

NanoPulse is distributed under the Apache-2.0 license.
