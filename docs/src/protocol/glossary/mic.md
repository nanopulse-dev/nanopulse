# MIC

The _MIC_ field contains a message integrity code over `Prefix || Frame` bytes
(excluding the MIC field) and is calculated using a MIC Key. It uses HMAC and
the SHA-256 hashing function. From the result, the first 4 bytes are taken and
used as MIC.

## Prefix

The prefix is defined as follow:

| 1 byte     | 1 byte      |
| ---------- | ----------- |
| Tx Channel | Tx Datarate |

## Rust example

```rust
fn calculate_mic(mic_key: &[u8; 32], bytes: &[u8], tx_ch: u8, tx_dr: u8) -> [u8; 4] {
    let mut mac = Hmac::<Sha256>::new_from_slice(mic_key).unwrap();
    mac.update(&[tx_ch, tx_dr]);
    mac.update(frame_bytes);

    let result = mac.finalize().into_bytes();
    result[0..4].try_into().unwrap()
}
```
