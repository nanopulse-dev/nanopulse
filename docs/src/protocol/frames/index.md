# Frames

The NanoPulse protocol consists of multiple frame types (up to 16). Each frame
starts with a 1 byte Frame Header containing the _Frame Type_ (4 upper bits) and
flags specific to the _Frame Type_ (lower 4 bits). The _Frame Header_ is
followed by a Payload and an optional MIC.

Bytes:

| 1 byte       | N bytes | 4 bytes (optional) |
| ------------ | ------- | ------------------ |
| Frame Header | Payload | MIC (optional)     |

## Frame Header

Bits:

| 7..4       | 3..0                      |
| ---------- | ------------------------- |
| Frame Type | Frame Type specific flags |

### Frame Type

| Value  | Frame Type                                          | Direction        | MIC |
| ------ | --------------------------------------------------- | ---------------- | --- |
| `0x00` | [Key Exchange Request](./key-exchange-request.md)   | Device > Server  | No  |
| `0x01` | [Key Exchange Response](./key-exchange-response.md) | Server > Device  | Yes |
| `0x02` | [Activation Request](./activation-request.md)       | Device > Server  | Yes |
| `0x03` | [Activation Response](./activation-response.md)     | Server > Device  | Yes |
| `0x04` | MAC Configuration                                   | Device <> Server | Yes |
| `0x05` | MAC Command                                         | Device <> Server | Yes |
| `0x06` | [Telemetry](./telemetry.md)                         | Device > Server  | Yes |
| `0x07` | [State](./state.md)                                 | Device <> Server | Yes |
| `0x08` | Configuration                                       | Device <> Server | Yes |
| `0x09` | Command                                             | Device <> Server | Yes |
| `0x0A` | Binary (?)                                          | Device <> Server | Yes |
| `0x0B` | Fragmented Binary (?)                               | Device <> Server | Yes |
| `0x0C` | RFU                                                 |                  |     |
| `0x0D` | RFU                                                 |                  |     |
| `0x0E` | RFU                                                 |                  |     |
| `0x0F` | RFU                                                 |                  |     |

### Payload

Frame Type specific Payload.

### MIC

Frame Type specific Message Integrity Code.

> [!NOTE]
> Not all frame-types contain a _MIC_.
