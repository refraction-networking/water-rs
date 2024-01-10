# cli tool for using `water` library

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
1. To run the shadowsocks WATM:

   -  run the server side from the [official implementation](https://github.com/shadowsocks/shadowsocks-rust) with the following config:
       ```json
       {
           "server": "127.0.0.1",
           "server_port": 8388,
           
           // match the password in the config passed into WATER-cli [remove the comment when copying]
           "password": "WATERisAwesome!",

           // for now, this is a global variable set in the ShadowSocks WATM, will add the config for it later
           // the global var is set here: https://github.com/erikziyunchi/water-rs/blob/48716579a3ff69a5de5e4f69c47ff2a80470d96d/examples/water_bins/ss_client_wasm_v1/src/lib.rs#L2
           // [remove these comment lines when copying]
           "method": "chacha20-ietf-poly1305"
       }
       ```
       and run the server side with:
       ```shell
       cargo run --bin ssserver -- -c <where-you-stored-the-config-from-above>.json
       ```

   - then run the cli tool with the `ss_client_wasm`
       ```shell
       cargo run --bin water_cli -- --wasm-path demo_wasm/ss_client_wasm.wasm --entry-fn v1_listen --config-wasm demo_configs/ss_config.json --type-client 3
       ```

   - to test the traffic is going through
       ```shell
       curl -4 -v --socks5 localhost:8080 https://erikchi.com
       ```

2. To run the v0_plus WATM:

   1. Run a v0_plus Listener:
       - use the cli tool
            ```shell
            cargo run --bin water_cli -- --wasm-path demo_wasm/plain.wasm --entry-fn _water_worker --config-wasm demo_configs/v0_listener_config.json --type-client 1
            ```

       - then test with:
         ```shell
         nc 127.0.0.1 8888
         hello
         hello
         ...
         ```
         you can also look at the log printed out by the cli tool to see the listener is receiving the input.

    2. Run a v0_plus Relay:
       - first you need a listener / destination for the Relay, you can use the above Listener for as it, then config the correct `ip:port` for the `remote` in the config file `demo_configs/v0_relay_config.json`, then run the cli tool:
            ```shell
            cargo run --bin water_cli -- --wasm-path demo_wasm/plain.wasm --entry-fn _water_worker --config-wasm demo_configs/v0_relay_config.json --type-client 2
            ```

       - then test with:
         ```shell
         nc 127.0.0.1 8080
         hello
         hello
         ...
         ```
         you can also look at the log printed out by the cli tool / the listener to see the Relay is relaying the input.