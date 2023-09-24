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
    let mut connector = runtime::WATERStreamConnector::new(conf)?;

    let mut rs = connector.connect()?;
    // rs.connect(&connector.config)?;

    Ok(())
}