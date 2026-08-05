# Activation Request

The Device sends an _Activation Request_ to (re)activate on the network. Each
new activation resets its (security) session, meaning it must revert to its
default values.

> [!NOTE]
> The first the the Device sends an _Activation Request_, the Server is able to
> validate that the Device has received the _Key Exchange Response_ and that
> both ends share the same PIN. It should no longer accept _Key Exchange
> Requests_ from this device.

Bytes:

| 1 byte       | 4 bytes  | 4 bytes                    | 8 bytes                  | 4 bytes |
| ------------ | -------- | -------------------------- | ------------------------ | ------- |
| Frame Header | Short ID | Activation Request Counter | Profile Info (encrypted) | MIC     |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x02` | RFU  |

> [!NOTE]
> RFU bits can be used in future to negotiate key (derivation) scheme.

## Short ID

The _Short ID_ of the device. See also [Short ID](../glossary/short-id.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md).

## Profile Info

Bytes:

| 4 bytes   | 2 bytes    | 2 bytes    |
| --------- | ---------- | ---------- |
| Vendor ID | Profile ID | Version ID |

### Vendor ID

_Vendor ID_ as defined in the NanoPulse device-repository.

### Profile ID

The _Profile ID_ is an per _Vendor ID_ unique profile identifier.

### Version ID

The _Version ID_ defines the version of the _Profile ID_ (e.g. different
configuration or different firmware version). The combination of _Vendor _ID_,
_Profile ID_ and _Version ID_ is used to lookup the device and its capabilities
(region config, codec, etc.) from the device-repository.

### Encryption

The Profile Info field is encrypted before transmission using the Encryption Key
(see below). The `IV` for the encryption key is:

| 8 bytes    | 4 bytes                    |
| ---------- | -------------------------- |
| `8 x 0x00` | Activation Request Counter |

#### Encryption key

See also [Keys](../glossary/keys.md).

- PRK: `Root Key`

- Info:

| 1 byte                        | 1 byte              | 1 byte            |
| ----------------------------- | ------------------- | ----------------- |
| Key usage Encryption = `0x02` | Frame Type = `0x02` | Downlink = `0x01` |

## MIC

MIC calculated over all frame bytes, except the last 4 MIC bytes. See also
[MIC](../glossary/mic.md).

### MIC key

See also [Keys](../glossary/keys.md).

- PRK: `Root Key`

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key usage MIC = `0x01` | Frame Type = `0x02` | Tx Channel | Tx Data-rate |
