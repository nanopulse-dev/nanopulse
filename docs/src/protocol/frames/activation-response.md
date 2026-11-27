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

| 12..15 | 8..11            | 4..7            | 0..3            |
| ------ | ---------------- | --------------- | --------------- |
| RFU    | Data-rate offset | RX Window Delay | Channel-plan ID |

### Encryption

The Network Info field is encrypted before transmission. Please see:

- [Encryption](../glossary/encryption.md)
- [Keys](../glossary/keys.md)

## MIC

Please see:

- [MIC](../glossary/mic.md)
- [Keys](../glossary/keys.md)
