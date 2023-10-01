use std::io::{Read, Write};
use std::sync::Arc;

use clap::Parser;

use crate::globals::{WASM_PATH, MAIN, CONFIF_WASM_PATH};

// pub mod parser;
pub mod wasm_shared_config;

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
    #[arg(short, long, default_value_t = String::from(CONFIF_WASM_PATH))]
    config_wasm: String,

    /// Optional argument specifying the client_type, default to be Runner
    #[arg(short, long, default_value_t = 2)]
    type_client: u32,
}

pub struct Config {
    pub filepath: String,
    pub entry_fn: String,
    pub config_wasm: String,
    pub client_type: u32,
}

impl Config {
    pub fn from_args() -> Result<Self, anyhow::Error> {
        let args = Args::parse();

        Self::init(args.wasm_path, args.entry_fn, args.config_wasm, args.type_client)
    }

    pub fn init(wasm_path: String, entry_fn: String, config_wasm: String, client_type: u32) -> Result<Self, anyhow::Error> {
        Ok(Config {
            filepath: wasm_path,
            entry_fn: entry_fn,
            config_wasm: config_wasm,
            client_type: client_type,
        })
    }
}