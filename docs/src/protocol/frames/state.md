# State

The _State_ frame can be sent by both the Server as Device. Initially, the
Server sends a _State_ frame to the device indicating which state must change
(e.g. valve open / close). To confirm the reception, the Device must respond
with a _State_ frame with a copy of the state change.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes       | N bytes             | 4 bytes |
| ------------ | -------- | ------------- | ------------------- | ------- |
| Frame Header | Short ID | State Counter | Payload (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..1 | 0        |
| ------ | ---- | -------- |
| `0x07` | RFU  | Downlink |

### Downlink

Must be set on Server to Device communication.

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md). Please
note that each direction (uplink / downlink) has a separate counter, such that
there are two counters:

- State Counter Up
- State Counter Down

## State Payload

This fields holds the binary State Payload. Encoding / decoding is implemented
in the device-profile.

### Encryption

The State Payload is encrypted before transmission using the Encryption Key (see
below). The `IV` for the encryption key is:

| 8 bytes    | 4 bytes       |
| ---------- | ------------- |
| `8 x 0x00` | State Counter |

#### Encryption Key

See also [Keys](../glossary/keys.md).

- PRK: `Session Root Key`

- Info:

| 1 byte                        | 1 byte              | 1 byte                             |
| ----------------------------- | ------------------- | ---------------------------------- |
| Key usage Encryption = `0x02` | Frame Type = `0x07` | Uplink = `0x00`, Downlink = `0x01` |

## MIC

MIC calculated over the _Frame Header_ + _Payload_ bytes. See also
[MIC](../glossary/mic.md).

### MIC key

See also [Keys](../glossary/keys.md)

- PRK: `Session Root Key`

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key usage MIC = `0x01` | Frame Type = `0x07` | Tx Channel | Tx Data-rate |
