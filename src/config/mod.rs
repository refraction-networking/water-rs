use std::io::{Read, Write};
use std::sync::Arc;

use clap::Parser;

use crate::globals::{WASM_PATH, MAIN, CONFIF_WASM_PATH};

// pub mod parser;
pub mod sharedconfig;

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

    /// Optional argument specifying the .wasm file to load
    #[arg(short, long, default_value_t = String::from(CONFIF_WASM_PATH))]
    config_wasm: String,
}

pub struct Config {
    pub filepath: String,
    pub entry_fn: String,
    pub config_wasm: String
}

impl Config {
    // pub fn init() -> Result<Arc<Self>, anyhow::Error> {
    pub fn init() -> Result<Self, anyhow::Error> {
        let args = Args::parse();

        // let config = Arc::new(Config {
        //     filepath: args.wasm_path,
        //     entry_fn: args.entry_fn,
        //     config_wasm: args.config_wasm,
        // });

        Ok(Config {
            filepath: args.wasm_path,
            entry_fn: args.entry_fn,
            config_wasm: args.config_wasm,
        })
    }
}