pub mod wasm_shared_config;


pub struct WATERConfig {
    pub filepath: String,
    pub entry_fn: String,
    pub config_wasm: String,
    pub client_type: WaterBinType,
    pub debug: bool,
}

impl WATERConfig {
    pub fn init(
        filepath: String,
        entry_fn: String,
        config_wasm: String,
        client_type: WaterBinType,
        debug: bool,
    ) -> Result<Self, anyhow::Error> {
        Ok(WATERConfig {
            filepath,
            entry_fn,
            config_wasm,
            client_type,
            debug,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterBinType {
    Unknown,
    Wrap,
    Dial,
    Listen,
    Relay,
    Runner,
}

impl From<u32> for WaterBinType {
    fn from(num: u32) -> Self {
        match num {
            0 => WaterBinType::Dial,
            1 => WaterBinType::Listen,
            2 => WaterBinType::Runner,
            3 => WaterBinType::Wrap,
            4 => WaterBinType::Relay,
            _ => WaterBinType::Unknown,
        }
    }
}
