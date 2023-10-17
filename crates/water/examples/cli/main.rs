use water::*;
use water::globals::{WASM_PATH, MAIN, CONFIG_WASM_PATH};
mod cli;
use cli;

use tracing_subscriber;
use tracing::Level;
use clap::Parser;




#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// optional address on which to listen
    #[arg(short, long, default_value_t = String::from("127.0.0.1:9001"))]
    listen: String,

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
    #[arg(short, long, default_value_t = 2)]
    type_client: u32,

    /// Optional argument specifying the client_type, default to be Runner
    #[arg(short, long, default_value_t = false)]
    debug: bool,
}


impl From<Args> for WATERConfig {
    fn from(args: Args) -> Self {
        Self {
            filepath: args.wasm_path,
            entry_fn: args.entry_fn,
            config_wasm: args.config_wasm,
            client_type: args.type_client,
            debug: args.debug,
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    cli::parse_and_execute()
}
