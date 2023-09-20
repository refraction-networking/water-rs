mod cli;

use wasmable_transport;

fn main() -> Result<(), anyhow::Error> {
    cli::parse_and_execute()
}
