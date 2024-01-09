//! Configuration info for the loading .wasm binary
//!
//! Passed as command line arguments when used with cli tool
//!
//! Will have the similar feat as required in [issue#19](https://github.com/gaukas/water/issues/19) on the go-side.

pub mod wasm_shared_config;

/// WATER configuration
#[derive(Clone)]
pub struct WATERConfig {
    /// Path to the .wasm binary
    pub filepath: String,

    /// Entry function name
    pub entry_fn: String,

    /// Path to the configuration file for the WATM binary
    pub config_wasm: String,

    /// Type of the client -- currently support Dial, Listen, Relay, Runner
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

/// WATER client type: A enum of types of the client
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
