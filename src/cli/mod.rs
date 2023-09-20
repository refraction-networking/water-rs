use wasmable_transport::*;

use std::sync::Arc;


pub fn parse_and_execute() -> Result<(), anyhow::Error> {
    // Parse command-line arguments and execute the appropriate commands
    let conf = Config::init()?;
    
    execute(conf)
}