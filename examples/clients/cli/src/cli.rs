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
        WaterBinType::Listen => {
            water_client.listen().unwrap();
            water_client.accept().unwrap();
            water_client.cancel_with().unwrap();

            let handle_water = water_client.run_worker().unwrap();

            // taking input from terminal
            loop {
                let mut buf = vec![0; 1024];
                let res = water_client.read(&mut buf);

                if res.is_ok() {
                    let str_buf = String::from_utf8(buf).unwrap();
                    if str_buf.trim() == "exit" {
                        water_client.cancel().unwrap();
                        break;
                    }

                    println!("Received: {}", str_buf);
                } else {
                    println!("Error: {}", res.unwrap_err());
                }
            }
            
            match handle_water.join().unwrap() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Running _water_worker ERROR: {}", e);
                    return Err(anyhow::anyhow!("Failed to join _water_worker thread"));
                }
            };
        }
        WaterBinType::Relay => {
            water_client.relay().unwrap();
            water_client.associate().unwrap();
            water_client.cancel_with().unwrap();

            let handle_water = water_client.run_worker().unwrap();

            // taking input from terminal
            loop {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();

                if input.trim() == "exit" {
                    water_client.cancel().unwrap();
                    break;
                }

                water_client.write(input.as_bytes()).unwrap();

                let mut buf = vec![0; 1024];
                let res = water_client.read(&mut buf);
                
                if res.is_ok() {
                    println!("Received: {}", String::from_utf8_lossy(&buf));
                } else {
                    println!("Error: {}", res.unwrap_err());
                }
            }
            
            match handle_water.join().unwrap() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Running _water_worker ERROR: {}", e);
                    return Err(anyhow::anyhow!("Failed to join _water_worker thread"));
                }
            };
        }
        WaterBinType::Wrap => {}
        WaterBinType::Unknown => {}
    }

    Ok(())
}
