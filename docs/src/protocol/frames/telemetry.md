# Telemetry

The _Telemetry_ frame is sent by the Device when it wants to exchange telemetry
data with the Server.

> [!NOTE]
> NanoPulse considers values like sensor data (counters, gauges, percentages,
> ...) as telemetry. In some cases there might be an overlap with
> [State](./state.md). For example the state of a valve can be considered
> telemetry or state depending the type of device. As a rule of thumb, if it is
> observed it is telemetry, if can be controlled it is state.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes | N bytes             | 4 bytes |
| ------------ | -------- | ------- | ------------------- | ------- |
| Frame Header | Short ID | Counter | Payload (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x06` | RFU  |

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md). Please
note that each direction (uplink / downlink) has a separate counter, such that
there are two counters:

- Counter Up
- Counter Down

## Telemetry Payload

This field holds the binary Telemetry Payload. Encoding / decoding is
implemented in the device-profile.

### Encryption

The Telemetry Payload is encrypted before transmission. Please see:

- [Encryption](../glossary/encryption.md)
- [Keys](../glossary/keys.md)

## MIC

Please see:

- [MIC](../glossary/mic.md)
- [Keys](../glossary/keys.md)
