extern crate anyhow;
extern crate clap;
extern crate tracing;
extern crate tracing_subscriber;

extern crate water;

use tracing::Level;

mod cli;

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    cli::parse_and_execute()
}
