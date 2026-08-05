# Activation Response

The _Activation Response_ is sent by the Server after receiving a valid
_Activation Request_ from a Device. The Server must resets the Device session to
its default values.

Bytes:

| 1 byte       | 4 bytes                     | 2 bytes                  | 4 bytes |
| ------------ | --------------------------- | ------------------------ | ------- |
| Frame Header | Activation Response Counter | Network Info (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x03` | RFU  |

> [!NOTE]
> Bits can be used in future to negotiate key (derivation) scheme.

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md).

## Network Info

Bits:

| 12..15 | 8..11            | 4..7       | 0..3            |
| ------ | ---------------- | ---------- | --------------- |
| RFU    | Data-rate offset | Data Delay | Channel-plan ID |

### Encryption

The Network Info field is encrypted before transmission using the Encryption Key
(see below). The IV for the encryption key is:

| 8 bytes    | 4 bytes                     |
| ---------- | --------------------------- |
| `8 x 0x00` | Activation Response Counter |

#### Encryption key

See also [Keys](../glossary/keys.md).

- PRK: `Session Root Key`

- Info:

| 1 byte                        | 1 byte              | 1 byte            |
| ----------------------------- | ------------------- | ----------------- |
| Key usage Encryption = `0x02` | Frame Type = `0x03` | Downlink = `0x01` |

## MIC

MIC calculated over all frame bytes, except the last 4 MIC bytes. See also
[MIC](../glossary/mic.md).

### MIC key

See also [Keys](../glossary/keys.md).

- PRK: `Session Root Key`

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key usage MIC = `0x01` | Frame Type = `0x03` | Tx Channel | Tx Data-rate |
