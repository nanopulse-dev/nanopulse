# Endianness

NanoPulse encodes multi-byte data-types as little-endian when encoding the
frames. Data-types that represent a sequences of bytes are not affected by this
convention. When a sequence of bits are documented, bit `0` means the
least-significant bit.

## Examples

### Encoded as little-endian

- Numeric nonce values
- Counter values

### Treated as sequences of bytes

- Short ID
- Public Key
- MIC
