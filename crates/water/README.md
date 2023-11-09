# Host Library -- `water`

A library for integrating WATER, satisfies APIs [here](https://app.gitbook.com/o/KHlQypYtIQKkb8YeZ6Hx/s/lVX5MqollomuX6vW80T6/rust-apis).

## Designs
**execute**: 
1. wasmtime runtime creation
2. Setup env:
    1. memory initialiation & limitation
    2. (`v1_preview` feature) wasm_config sharing to WASM
    3. export helper functions (e.g. creation of TCP, TLS, crypto, etc)
3. (`v1` feature) setup multi-threading
4. Run the `entry_fn` or execute as the Role (`Dial`, `Listen`, `Relay`)