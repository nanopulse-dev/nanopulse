# MIC

The _MIC_ field contains a message integrity code over the Frame bytes
(excluding the MIC field) and is calculated using a MIC Key. It uses HMAC and
the SHA-256 hashing function. From the result, the first 4 bytes are taken and
used as MIC.

## Rust example

```rust
fn calculate_mic(mic_key: &[u8; 32], bytes: &[u8]) -> [u8; 4] {
    let mut mac = Hmac::<Sha256>::new_from_slice(mic_key).unwrap();
    mac.update(frame_bytes);

    let result = mac.finalize().into_bytes();
    result[0..4].try_into().unwrap()
}
```
