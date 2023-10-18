use water::globals::{CONFIG_WASM_PATH, MAIN, WASM_PATH};
use water::{config::WATERConfig, runtime};

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

pub fn parse() -> Result<WATERConfig, anyhow::Error> {
    // Parse command-line arguments and execute the appropriate commands

    let conf: WATERConfig = Args::parse().into();
    Ok(conf)
}

pub fn parse_and_execute() -> Result<(), anyhow::Error> {
    execute(parse()?)
}

pub fn execute(conf: WATERConfig) -> Result<(), anyhow::Error> {
    let mut water_client = runtime::WATERClient::new(conf)?;

    // // FIXME: hardcoded the addr & port for now
    // water_client.connect("", 0)?;

    // loop {
    //     // keep reading from stdin and call read and write function from water_client.stream
    //     let mut buf = String::new();
    //     std::io::stdin().read_line(&mut buf)?;

    //     water_client.write(buf.as_bytes())?;

    //     let mut buf = vec![0; 1024];
    //     water_client.read(&mut buf)?;

    //     println!("read: {:?}", String::from_utf8_lossy(&buf));
    // }

    Ok(())
}
