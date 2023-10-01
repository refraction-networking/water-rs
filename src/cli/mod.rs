use crate::{config::Config, runtime};

use std::sync::Arc;

pub fn parse() -> Result<Config, anyhow::Error> {
    // Parse command-line arguments and execute the appropriate commands
    let conf = Config::from_args()?;
    Ok(conf)
}

pub fn parse_and_execute() -> Result<(), anyhow::Error> {
    execute(parse()?)
}

pub fn execute(conf: Config) -> Result<(), anyhow::Error> {
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