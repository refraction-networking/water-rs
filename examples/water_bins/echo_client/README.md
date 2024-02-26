# WATM as worker implementation of an echo client

It uses the `water-wasm-crate` library and is able to run as a `Dialer` or `Listener` Role. It is currently implemented with `v1_preview` where it has the capability of managing connections inside it (tell the Host which `address:port` it `dial()` to / `listen()` on).

## How to run?
You can also find a full runnable test for this example with using our water-host library [here](https://github.com/refraction-networking/water-rs/blob/main/tests/tests/echo_tests.rs).

You can config it as the following:
```json
{
    "remote_address": "127.0.0.1",
    "remote_port": 8080,
    "local_address": "127.0.0.1",
    "local_port": 8088
}
```

## How to compile?
```shell
cargo build --target wasm32-wasi && mv ../../../target/wasm32-wasi/debug/echo_client.wasm ./echo_client.wasm
```

If you want to optmize the size of the `.wasm` binary:
```shell
wasm-opt --strip-debug ./echo_client.wasm -o ./echo_client.wasm
```