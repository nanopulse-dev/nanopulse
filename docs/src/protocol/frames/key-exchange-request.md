# Key Exchange Request

The Device sends a _Key Exchange Request_ to exchange its _Device Public Key_
with the Server, and to receive the _Server Public Key_. It only performs this
request when the _Server Public Key_ is unknown to the device. Once it has
succesfully completed the _Key Exchange_, the Device must persist the _Server
Public Key_ in its memory such that after a power-cycle or reset, it can
immediately proceed with an _Activation Request_.

Bytes:

| 1 byte       | 32 bytes          | 4 bytes |
| ------------ | ----------------- | ------- |
| Frame Header | Device Public Key | Nonce   |

## Frame Header

Bits:

| 7..4   | 3..0 |
| ------ | ---- |
| `0x00` | RFU  |

> [!NOTE]
> RFU bits can be used in future to negotiate key (derivation) scheme.

## Device Public Key

The public-key of the device. See also [Keys](../glossary/keys.md).

## Nonce

Random `u32` nonce.

> [!NOTE]
> As the Server can not authenticate the content of the _Key Exchange Request_,
> this value must be random.
