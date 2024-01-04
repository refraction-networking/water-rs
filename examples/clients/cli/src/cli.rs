use water::config::{WATERConfig, WaterBinType};
use water::globals::{CONFIG_WASM_PATH, MAIN, WASM_PATH};
use water::runtime;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional argument specifying the .wasm file to load
    #[arg(short, long, default_value_t = String::from(WASM_PATH))]
    wasm_path: String,

    /// Optional argument specifying name of the function in the .wasm file to use
    #[arg(short, long, default_value_t = String::from(MAIN))]
    entry_fn: String,

    /// Optional argument specifying the config file
    #[arg(short, long, default_value_t = String::from(CONFIG_WASM_PATH))]
    config_wasm: String,

    /// Optional argument specifying the client_type, default to be Runner
    #[arg(short, long, default_value_t = 3)]
    type_client: u32,

    /// Optional argument enabling debug logging
    #[arg(short, long, default_value_t = true)]
    debug: bool,
}

impl From<Args> for WATERConfig {
    fn from(args: Args) -> Self {
        Self {
            filepath: args.wasm_path,
            entry_fn: args.entry_fn,
            config_wasm: args.config_wasm,
            client_type: WaterBinType::from(args.type_client),
            debug: args.debug,
        }
    }
}

pub fn parse() -> Result<WATERConfig, anyhow::Error> {
    // Parse command-line arguments and execute the appropriate commands

    let conf: WATERConfig = Args::parse().into();
    Ok(conf)
}

pub fn parse_and_execute() -> Result<(), anyhow::Error> {
    execute(parse()?)
}

pub fn execute(_conf: WATERConfig) -> Result<(), anyhow::Error> {
    let mut water_client = runtime::client::WATERClient::new(_conf).unwrap();

    match water_client.config.client_type {
        WaterBinType::Dial => {
            water_client.connect().unwrap();
        }
        WaterBinType::Runner => {
            water_client.execute().unwrap();
        }
        WaterBinType::Listen => {}
        WaterBinType::Relay => {}
        WaterBinType::Wrap => {}
        WaterBinType::Unknown => {}
    }

    Ok(())
}
