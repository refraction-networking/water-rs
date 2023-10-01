use water::*;
use rand;

use pprof::protos::Message;
use std::io::Write;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tracing_subscriber;
use tracing::Level;

#[test]
fn benchmarking_v0() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Start the listener in a new thread
    let listener_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
        let conf = config::Config::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("listen"), String::from("./tests/test_data/config.json"), 2)?;

        let mut water_client = runtime::WATERClient::new(conf)?;

        water_client.execute()
    });

    // Give the listener some time to start
    std::thread::sleep(std::time::Duration::from_millis(5000));

    // --------- start to dial the listener ---------
    let dial_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
        let conf = config::Config::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("dial"), String::from("./tests/test_data/config.json"), 0)?;
    
        let mut water_client = runtime::WATERClient::new(conf)?;
    
        // FIXME: hardcoded the addr & port for now
        water_client.connect("", 0)?;
    
        let guard = pprof::ProfilerGuard::new(100).unwrap();
    
        for _ in 0..10000 {
            let random_data: Vec<u8> = (0..1000).map(|_| rand::random::<u8>()).collect();
    
            water_client.write(&random_data)?;
    
            let mut buf = vec![0; 1000];
            water_client.read(&mut buf)?;
    
            // println!("read: {:?}", String::from_utf8_lossy(&buf));
        }
    
        // Stop and report profiler data
        if let Ok(report) = guard.report().build() {
            // println!("{:?}", report);
            // report.flamegraph(std::io::stdout())?;
            let mut file = std::fs::File::create("flamegraph.svg")?;
            report.flamegraph(file)?;
    
            // let mut file = std::fs::File::create("profile.pb")?;
            // report.pprof(file)?;
            let mut file = std::fs::File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();
    
            let mut content = Vec::new();
            // profile.encode(&mut content).unwrap();
            profile.write_to_vec(&mut content).unwrap();
            file.write_all(&content).unwrap();
        }

        Ok(())
    });

    dial_handle.join().expect("Listener thread panicked")?;

    // // Signal the listener thread to stop
    // should_stop.store(true, Ordering::Relaxed);

    Ok(())
}