pub mod wasm_shared_config;

pub struct WATERConfig {
    pub filepath: String,
    pub entry_fn: String,
    pub config_wasm: String,
    pub client_type: u32,
    pub debug: bool,
}

impl WATERConfig {
    pub fn init(wasm_path: String, entry_fn: String, config_wasm: String, client_type: u32, debug: bool) -> Result<Self, anyhow::Error> {
        Ok(WATERConfig {
            filepath: wasm_path,
            entry_fn: entry_fn,
            config_wasm: config_wasm,
            client_type: client_type,
            debug: debug,
        })
    }
}