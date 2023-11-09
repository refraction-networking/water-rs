# cli tool for using `water` library

🚧 Currently under reimplementation 🚧

## How to run?
To run the Host program + WASM:
```shell
cargo run --bin wasmable_transport -- --wasm-path <./proxy.wasm> --entry-fn <main> --config-wasm <conf.json>
```

Then you can netcat into the connection, for now, I included a `proxy.wasm` as a multiple conneciton echo server, test with several terminals:
```shell
nc 127.0.0.1 9005
```
you should see `> CONNECTED` in the terminal of running WASM, then you can connect a bunch like this and input anything to see how it echos.