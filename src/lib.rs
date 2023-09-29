pub mod config;
pub mod runtime;
pub mod errors;
pub mod utils;
pub mod globals;

use config::*;
// use runtime::{Host, WASMRuntime};

use std::sync::Arc;

// Re-export main components for easier access
// pub use wasmruntime::{RuntimeConn, RuntimeDialer, RuntimeDialerConn};
pub use config::Config;

pub fn execute(conf: Config) -> Result<(), anyhow::Error> {
    let mut water_client = runtime::WATERClient::new(conf)?;

    // FIXME: hardcoded the addr & port for now
    water_client.connect("", 0)?;

    loop {
        // keep reading from stdin and call read and write function from water_client.stream
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        
        water_client.stream.write(buf.as_bytes())?;
        
        let mut buf = vec![0; 1024];
        water_client.stream.read(&mut buf)?;

        println!("read: {:?}", String::from_utf8_lossy(&buf));
    }

    Ok(())
}