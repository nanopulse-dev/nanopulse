# Key Exchange Response

The Server sends a _Key Exchange Response_ if:

- It accepts the Device Public Key and knows its PIN
- The Device has not yet send its first
  [Activation Request](./activation-request.md)

In any other case, it must not respond with a _Key Exchange Response_.

> [!NOTE]
> Receiving the first _Activation Request_ completes the _Key Exchange_, as it
> confirms that the Device has received the _Key Exchange Response_.

Bytes:

| 1 byte       | 32 bytes          | 4 bytes                       | 4 bytes |
| ------------ | ----------------- | ----------------------------- | ------- |
| Frame Header | Server Public Key | Key Exchange Response Counter | MIC     |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x01` | RFU  |

> [!NOTE]
> RFU bits can be used in future to negotiate key (derivation) scheme.

## Server Public Key

The public-key of the server. See also [Keys](../glossary/keys.md).

## Counter

Unique `u32` counter value. See also [Counter](../glossary/counter.md).

## MIC

MIC calculated over all frame bytes, except the last 4 MIC bytes. See also
[MIC](../glossary/mic.md).

### MIC Key

See also [Keys](../glossary/keys.md).

- PRK: `Root Key`.

- Info:

| 1 byte                 | 1 byte              | 1 byte     | 1 byte       |
| ---------------------- | ------------------- | ---------- | ------------ |
| Key Usage MIC = `0x01` | Frame Type = `0x01` | Tx Channel | Tx Data-rate |
