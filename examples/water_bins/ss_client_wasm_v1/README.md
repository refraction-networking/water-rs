# WASM as manager implementation of ShadowSocks-Client

It uses the `wasm-water-crate` library and manages the whole flow of the ShadowSocks Client, to test it for now, you need to also deploy the ShadowSocks's [native server](https://github.com/shadowsocks/shadowsocks-rust) with the same configuration using here.

Currently the config we have written into the client is:
```json
{
    "server": "127.0.0.1",
    "server_port": 8388,
    "password": "Test!23",
    "method": "chacha20-ietf-poly1305",
}
```

