mod cli;

use wasmable_transport;
use tracing_subscriber;
use tracing::Level;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    cli::parse_and_execute()
}
