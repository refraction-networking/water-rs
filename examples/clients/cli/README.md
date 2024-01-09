# cli tool for using `water` library

🚧 Currently under reimplementation 🚧

## How to run?
To run the Host program + WASM:
```shell
cargo run --bin water_cli -- --wasm-path <./proxy.wasm> --entry-fn <main> --config-wasm <config.json> --type-client <3>
```

Then you can netcat into the connection, for now, I included a `proxy.wasm` as a multiple conneciton echo server, test with several terminals:
```shell
nc 127.0.0.1 9005
```
you should see `> CONNECTED` in the terminal of running WASM, then you can connect a bunch like this and input anything to see how it echos.

## Examples
To run the shadowsocks wasm:

1. run the server side from the [official implementation](https://github.com/shadowsocks/shadowsocks-rust) with the following config:
    ```json
    {
        "server": "127.0.0.1",
        "server_port": 8388,
        "password": "Test!23",
        "method": "chacha20-ietf-poly1305"
    }
    ```
    and run the server side with:
    ```shell
    cargo run --bin ssserver -- -c <where-you-stored-the-config-from-above>.json
    ```

2. then run the cli tool with the `ss_client_wasm`
    ```shell
    cargo run --bin water_cli -- --wasm-path demo_wasm/ss_client_wasm.wasm --entry-fn v1_listen --config-wasm demo_configs/ss_config.json --type-client 3
    ```

3. to test the traffic is going through
    ```shell
    curl -4 -v --socks5 localhost:8080 https://erikchi.com
    ```
