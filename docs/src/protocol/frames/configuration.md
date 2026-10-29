# Configuration

The _Configuration_ frame is used to request a configuration change (e.g.
configuring the telemetry interval).The request is typically sent by the
_Server_ to the _Device_, and the acknowledgement is typically sent by the
_Device_ to the _Server_. The request must be interpreted as a single
transaction such that all changes either pass or fail.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes | N bytes             | 4 bytes |
| ------------ | -------- | ------- | ------------------- | ------- |
| Frame Header | Short ID | Counter | Payload (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..2 | 1   | 0         |
| ------ | ---- | --- | --------- |
| `0x08` | RFU  | Ack | Direction |

### Direction

- `0` = Uplink
- `1` = Downlink

### Ack

- `0`: The _Payload_ field contains the request.
- `1`: The _Payload_ field contains the response.

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md). Please
note that each direction (uplink / downlink) has a separate counter, such that
there are two counters:

- Counter Up
- Counter Down

## Payload

Depending the _Ack_ bit-field, this payload must either be interpreted as a
request or as an acknowledgement.

### Request

Bytes:

| N bytes         |
| --------------- |
| Request Payload |

Encoding / decoding of the _Request Payload_ is implemented in the
device-profile.

### Acknowledgement

Bytes:

| 2 bytes         | N bytes                 |
| --------------- | ----------------------- |
| Request Counter | Acknowledgement Payload |

#### Request Counter

The _Request Counter_ contains the 16LSB of the _Counter_ value of the frame
containing the _Request Payload_ to which the acknowledgement is related.

#### Acknowledgement Payload

In case of success, this field is empty. In any other case, this field contains
the status payload that can be decoded by the device-profile to one or multiple
error messages.

### Encryption

The Payload is encrypted before transmission using the Encryption Key (see
below). The `IV` for the encryption key is:

| 8 bytes    | 4 bytes |
| ---------- | ------- |
| `8 x 0x00` | Counter |

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
