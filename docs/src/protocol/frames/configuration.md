# Configuration

The _Configuration_ frame can be sent by both the Server and Device. Initially,
the Server sends a _Configuration_ frame to the device indicating which
configuration must change (e.g. telemetry interval). To confirm the reception,
the Device must respond with a _Configuration_ frame with a copy of the
configuration change.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes               | N bytes             | 4 bytes |
| ------------ | -------- | --------------------- | ------------------- | ------- |
| Frame Header | Short ID | Configuration Counter | Payload (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..1 | 0        |
| ------ | ---- | -------- |
| `0x08` | RFU  | Downlink |

### Downlink

Must be set on Server to Device communication.

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md). Please
note that each direction (uplink / downlink) has a separate counter, such that
there are two counters:

- Configuration Counter Up
- Configuration Counter Down

## Configuration Payload

This fields holds the binary Configuration Payload. Encoding / decoding is
implemented in the device-profile.

### Encryption

The Configuration Payload is encrypted before transmission using the Encryption
Key (see below). The `IV` for the encryption key is:

| 8 bytes    | 4 bytes               |
| ---------- | --------------------- |
| `8 x 0x00` | Configuration Counter |

#### Encryption Key

See also [Keys](../glossary/keys.md).

- PRK: `Session Root Key`

- Info:

| 1 byte                        | 1 byte              | 1 byte                             |
| ----------------------------- | ------------------- | ---------------------------------- |
| Key usage Encryption = `0x02` | Frame Type = `0x08` | Uplink = `0x00`, Downlink = `0x01` |

## MIC

MIC calculated over the _Frame Header_ + _Payload_ bytes. See also
[MIC](../glossary/mic.md).

### MIC key

See also [Keys](../glossary/keys.md)

- PRK: `Session Root Key`

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key usage MIC = `0x01` | Frame Type = `0x08` | Tx Channel | Tx Data-rate |
