# NanoPulse - open-source IoT toolkit

> [!WARNING]
> This project is work-in-progress, the documentation, protocol and software are
> still in development. Documentation is incomplete and subject to change.

NanoPulse is an open-source toolkit for building, connecting and integrating
low-power IoT devices. The NanoPulse protocol is PHY-layer agnostic, and
therefore can work with various chipsets and modulations (currently SX1302/3 for
the NanoPulse Gateways and SX126X for the NanoPulse Device SDK, support for the
CC1101 is planned).

## Key features

### Curve25519 / Diffie-Hellman

NanoPulse uses Curve25519 and Diffie-Hellman based cryptography and therefore
does not rely on pre-shared-keys. NanoPulse provides a Key Exchange mechanism to
let the device exchange its public key with the server in a secure and easy way.

### Configuration and state synchronization

The NanoPulse Server keeps a copy of the state and configuration each device,
containing the actual and desired state and configuration of the device and will
re-synchronize automatically when needed.

### Profiles

On activation, each device announces its profile information, which maps to a
profile configuration in the NanoPulse Server. This profile contains the region
configuration, codec functions for decoding telemetry data and functions for
encoding / decoding state and configuration changes.

### Home Assistant

NanoPulse provides out-of-the-box integration with Home Assistant. NanoPulse can
also be installed as a Home Assistant add-on. NanoPulse based devices will be
automatically recognized by Home Assistant, and can be used with any other Home
Assistant integration.

## Components

- NanoPulse protocol
- NanoPulse Gateway service
- NanoPulse Server service
- NanoPulse Device SDK

### NanoPulse protocol

The NanoPulse protocol is an easy to understand and lightweight protocol. It
uses Curve25519 / Diffie-Hellman based cryptography, taking away the need to use
pre-shared-keys. The protocol implements initial Key Exchange (for public-key
exchange between the device and server), Activation (to activate the device) and
Data-transport. On Activation, the device automatically advertises its profile
reducing the amount of configuration that must be performed when provisioning
the Device on the Server.

### NanoPulse Gateway service

The NanoPulse Gateway software is used for setting up one or multiple NanoPulse
Gateways (powered by the SX1302 or CC1101 chipset). NanoPulse Gateways are
network connected with the NanoPulse Server.

### NanoPulse Server service

NanoPulse Server is the central brain where devices on the network are managed.
It handles the NanoPulse MAC-layer, cryptography and integrations with
third-party applications like Home Assistant.

### NanoPulse Device SDK

NanoPulse provides an SDK for building devices based on the NanoPulse protocol,
including several examples. The full SDK is implemented in Rust using Embassy
(which supports STM32, nRF, RP2040 and ESP32). The NanoPulse Device SDK itself
provides various abstractions, to make it easy to add additional radio chipsets.
Currently it supports SX126X and CC1101 is planned.
