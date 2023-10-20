// use cap_std::net::TcpStream;
use water::*;
// use rand;

// use pprof::protos::Message;
// use std::net::{TcpListener, TcpStream};
// use std::thread;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;

use tracing::Level;

// use std::time::Instant;
// use tracing::info;

// use std::io::{Read, Write, ErrorKind};
// use std::thread::sleep;
// use std::time::Duration;

#[test]
fn wasm_managed_shadowsocks_async() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let conf = config::WATERConfig::init(
        String::from("./test_wasm/ss_client_wasm.wasm"),
        String::from("ss_client_execute"),
        String::from("./test_data/config.json"),
        2,
        true,
    )
    .unwrap();

    let mut water_client = runtime::WATERClient::new(conf).unwrap();
    water_client.execute().unwrap();
}

// #[test]
// fn SS_handler_testing() {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::INFO)
//         .init();

//     let listener = TcpListener::bind("127.0.0.1:1080").expect("Failed to bind to address");
//     println!("Listening on {:?}", listener.local_addr().unwrap());
//     for stream in listener.incoming() {
//         match stream {
//             Ok(client) => {
//                 // handle onely 1 client
//                 handle_client(client);
//             }
//             Err(e) => {
//                 println!("Error accepting client: {}", e);
//             }
//         }
//     }
// }

// this is the test where SOCKS5 server + listener is at the Host -- V0
// #[test]
// fn SS_client_no_socks5() -> Result<(), anyhow::Error> {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::INFO)
//         .init();

//     // --------- start to dial the listener ---------
//     let dial_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
//         // Measure initialization time
//         let conf = config::WATERConfig::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("_dial"), String::from("./tests/test_data/config.json"), 0, true)?;
//         let mut water_client = runtime::WATERClient::new(conf)?;
//         water_client.connect("", 0)?;

//         // let mut water_client = TcpStream::connect(("127.0.0.1", 8088))?;

//         // Not measuring the profiler guard initialization since it's unrelated to the read/write ops
//         let guard = pprof::ProfilerGuard::new(100).unwrap();

//         let single_data_size = 1024; // Bytes per iteration
//         let total_iterations = 1;

//         let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

//         let start = Instant::now();
//         for _ in 0..total_iterations {
//             water_client.write(&random_data)?;

//             let mut buf = vec![0; single_data_size];
//             water_client.read(&mut buf)?;
//         }

//         let elapsed_time = start.elapsed().as_secs_f64();
//         let total_data_size_mb = (total_iterations * single_data_size) as f64;
//         let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

//         info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

//         let single_data_size = 1024; // Bytes per iteration
//         let total_iterations = 100;

//         let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

//         let start = Instant::now();
//         for _ in 0..total_iterations {
//             water_client.write(&random_data)?;

//             let mut buf = vec![0; single_data_size];
//             water_client.read(&mut buf)?;
//         }

//         let elapsed_time = start.elapsed().as_secs_f64();
//         let total_data_size_mb = (total_iterations * single_data_size) as f64;
//         let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

//         info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

//         // Stop and report profiler data
//         if let Ok(report) = guard.report().build() {
//             // println!("{:?}", report);
//             // report.flamegraph(std::io::stdout())?;
//             let mut file = std::fs::File::create("flamegraph.svg")?;
//             report.flamegraph(file)?;

//             // let mut file = std::fs::File::create("profile.pb")?;
//             // report.pprof(file)?;
//             let mut file = std::fs::File::create("profile.pb").unwrap();
//             let profile = report.pprof().unwrap();

//             let mut content = Vec::new();
//             // profile.encode(&mut content).unwrap();
//             profile.write_to_vec(&mut content).unwrap();
//             file.write_all(&content).unwrap();
//         }

//         Ok(())
//     });

//     dial_handle.join().expect("Listener thread panicked")?;

//     // // Signal the listener thread to stop
//     // should_stop.store(true, Ordering::Relaxed);

//     Ok(())
// }
