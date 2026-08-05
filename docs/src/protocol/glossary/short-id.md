# Short ID

The _Short ID_ is derived from the Public Key by taking the first 4 bytes of the
`SHA256(public_key)`. See also [Keys](./keys.md).

> [!WARNING]
> The _Short ID_ is not guaranteed to be unique and should not be used as a
> primary key. However, it should be used as an index to find devices matching
> the _Short ID_, after which the correct device can be identified using the
> _Signature_.
