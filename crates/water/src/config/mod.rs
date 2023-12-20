pub mod wasm_shared_config;

use std::fmt;
use std::str::FromStr;

/// WATER configuration
#[derive(Clone)]
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

/// WATER client type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WaterBinType {
    Dial,
    Listen,
    Relay,
    Runner,
    Wrap,
    Unknown,
}

impl From<u32> for WaterBinType {
    fn from(num: u32) -> Self {
        match num {
            0 => WaterBinType::Dial,
            1 => WaterBinType::Listen,
            2 => WaterBinType::Relay,
            3 => WaterBinType::Runner,
            4 => WaterBinType::Wrap,
            _ => WaterBinType::Unknown,
        }
    }
}
