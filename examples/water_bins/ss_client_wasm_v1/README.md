# WATM as manager implementation of ShadowSocks-Client

It uses the `water-wasm-crate` library and manages the whole flow of the ShadowSocks Client as an example(ported from [ShadowSocks' official Rust implementation](https://github.com/shadowsocks/shadowsocks-rust)) of porting existing proxy protocol implementations, to test it for now, you need to also deploy the ShadowSocks's [native server](https://github.com/shadowsocks/shadowsocks-rust) with the same configuration using here. It is currently implemented with `v1_preview`.

## How to run?
You can also find a full runnable test for this example with using our water-host library [here](https://github.com/erikziyunchi/water-rs/blob/main/tests/tests/ss_testing.rs).

Currently the config we have written into the client is:
```json
{
    "server": "127.0.0.1",
    "server_port": 8388,
    "password": "Test!23",
    "method": "chacha20-ietf-poly1305",
}
```

## How to compile?
```shell
cargo build --target wasm32-wasi && mv ../../../target/wasm32-wasi/debug/ss_client_wasm.wasm ./ss_client_wasm.wasm
```

If you want to optmize the size of the `.wasm` binary:
```shell
wasm-opt --strip-debug ./ss_client_wasm.wasm -o ./ss_client_wasm.wasm
```