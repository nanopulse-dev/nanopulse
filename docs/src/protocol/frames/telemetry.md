# Telemetry

The _Telemetry_ frame is sent by the Device when it wants to exchange telemetry
data with the Server.

> [!NOTE]
> NanoPulse considers values like sensor data (counters, gauges, percentages,
> ...) as telemetry. In some cases there might be an overlap with state. For
> example the state of a valve can be considered telemetry or state depending
> the type of device. As a rule of thumb, if the Server can not control it, it
> is telemetry.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes           | N bytes             | 4 bytes |
| ------------ | -------- | ----------------- | ------------------- | ------- |
| Frame Header | Short ID | Telemetry Counter | Payload (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x06` | RFU  |

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md).

## Telemetry Payload

This field holds the binary Telemetry Payload. Encoding / decoding is
implemented in the device-profile.

### Encryption

The Telemetry Payload is encrypted before transmission using the Encryption Key
(see below). The `IV` for the encryption key is:

| 8 bytes    | 4 bytes           |
| ---------- | ----------------- |
| `8 x 0x00` | Telemetry Counter |

#### Encryption key

See also [Keys](../glossary/keys.md).

- PRK: `Session Root Key`

- Info:

| 1 byte                        | 1 byte              | 1 byte          |
| ----------------------------- | ------------------- | --------------- |
| Key usage Encryption = `0x02` | Frame Type = `0x06` | Uplink = `0x00` |

## MIC

MIC calculated over all frame bytes, except the last 4 MIC bytes. See also
[MIC](../glossary/mic.md).

### MIC key

See also [Keys](../glossary/keys.md)

- PRK: `Root Key`

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key usage MIC = `0x01` | Frame Type = `0x06` | Tx Channel | Tx Data-rate |
