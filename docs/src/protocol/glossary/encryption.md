# Encryption

Fields that are encrypted, are encrypted and decrypted using ChaCha20. Together
with the Encryption Key, an unique IV is being used to make sure that encrypting
the same plaintext never result in the same ciphertext.

## Rust example

> [!NOTE]
> Both `apply_keystream` is used for encryption and decryption. E.g.
> `apply_keystream` against a plaintext produces the ciphertext,
> `apply_keystream` against a ciphertext produces the ciphertext.

```rust
fn encrypt_decrypt_payload(enc_key: &[u8; 32], iv: &[u8; 12], bytes: &mut [u8]) {
    let mut cipher = ChaCha20::new(enc_key.into(), iv.into());
    cipher.apply_keystream(bytes);
}
```
