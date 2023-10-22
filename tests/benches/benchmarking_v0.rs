// use cap_std::net::TcpStream;
use water::*;

use pprof::protos::Message;
// use std::net::{TcpListener, TcpStream};
use std::net::TcpListener;
use std::thread;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;

use tracing::Level;

use std::time::Instant;
use tracing::info;

use std::io::{Read, Write};
// use std::io::{Read, Write, ErrorKind};
// use std::thread::sleep;
// use std::time::Duration;

#[test]
fn benchmarking_v0_echo() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        println!("Listening on {:?}", listener.local_addr().unwrap());

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = vec![0u8; 4096];

                    loop {
                        match stream.read(&mut buf) {
                            Ok(n) => {
                                if n == 0 {
                                    break; // Connection was closed.
                                }

                                // Echo data back to client.
                                if let Err(e) = stream.write_all(&buf[..n]) {
                                    eprintln!("Error writing to client: {:?}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading from client: {:?}", e);
                                break;
                            }
                        }
                    }

                    println!("Connection closed.");
                }
                Err(e) => {
                    eprintln!("Connection failed: {:?}", e);
                }
            }
        }
    });

    // // Give the listener some time to start
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // --------- start to dial the listener ---------
    let dial_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
        // Measure initialization time
        let conf = config::WATERConfig::init(
            String::from("./tests/test_wasm/proxy.wasm"),
            String::from("_dial"),
            String::from("./tests/test_data/config.json"),
            config::WaterBinType::Dial,
            true,
        )?;
        let mut water_client = runtime::WATERClient::new(conf)?;
        water_client.connect("", 0)?;

        // let mut water_client = TcpStream::connect(("127.0.0.1", 8088))?;

        // Not measuring the profiler guard initialization since it's unrelated to the read/write ops
        let guard = pprof::ProfilerGuard::new(100).unwrap();

        let single_data_size = 1024; // Bytes per iteration
        let total_iterations = 1;

        let random_data: Vec<u8> = (0..single_data_size)
            .map(|_| rand::random::<u8>())
            .collect();

        let start = Instant::now();
        for _ in 0..total_iterations {
            water_client.write(&random_data)?;

            let mut buf = vec![0; single_data_size];
            water_client.read(&mut buf)?;
        }

        let elapsed_time = start.elapsed().as_secs_f64();
        let total_data_size_mb = (total_iterations * single_data_size) as f64;
        let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

        info!(
            "avg bandwidth: {:.2} MB/s (N={})",
            avg_bandwidth, total_iterations
        );

        let single_data_size = 1024; // Bytes per iteration
        let total_iterations = 100;

        let random_data: Vec<u8> = (0..single_data_size)
            .map(|_| rand::random::<u8>())
            .collect();

        let start = Instant::now();
        for _ in 0..total_iterations {
            water_client.write(&random_data)?;

            let mut buf = vec![0; single_data_size];
            water_client.read(&mut buf)?;
        }

        let elapsed_time = start.elapsed().as_secs_f64();
        let total_data_size_mb = (total_iterations * single_data_size) as f64;
        let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

        info!(
            "avg bandwidth: {:.2} MB/s (N={})",
            avg_bandwidth, total_iterations
        );

        // ================== test more iterations ==================
        // let single_data_size = 1024; // Bytes per iteration
        // let total_iterations = 10000;

        // let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

        // let start = Instant::now();
        // for _ in 0..total_iterations {
        //     water_client.write(&random_data)?;

        //     let mut buf = vec![0; single_data_size];
        //     water_client.read(&mut buf)?;
        // }

        // let elapsed_time = start.elapsed().as_secs_f64();
        // let total_data_size_mb = (total_iterations * single_data_size) as f64;
        // let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

        // info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

        // let single_data_size = 1024; // Bytes per iteration
        // let total_iterations = 43294;

        // let random_data: Vec<u8> = (0..single_data_size).map(|_| rand::random::<u8>()).collect();

        // let start = Instant::now();
        // for _ in 0..total_iterations {
        //     water_client.write(&random_data)?;

        //     let mut buf = vec![0; single_data_size];
        //     water_client.read(&mut buf)?;
        // }

        // let elapsed_time = start.elapsed().as_secs_f64();
        // let total_data_size_mb = (total_iterations * single_data_size) as f64;
        // let avg_bandwidth = total_data_size_mb / elapsed_time / 1024.0 / 1024.0;

        // info!("avg bandwidth: {:.2} MB/s (N={})", avg_bandwidth, total_iterations);

        // Stop and report profiler data
        if let Ok(report) = guard.report().build() {
            // println!("{:?}", report);
            // report.flamegraph(std::io::stdout())?;
            let file = std::fs::File::create("flamegraph.svg")?;
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

// ==================== test for async listener for V1 ====================
// #[test]
// fn test_async_listener() {
//     // Start the listener in a new thread
//     let listener_handle = std::thread::spawn(|| -> Result<(), anyhow::Error> {
//         // let conf = config::WATERConfig::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("listen"), String::from("./tests/test_data/config.json"), 2)?;

//         // let mut water_client = runtime::WATERClient::new(conf)?;

//         // water_client.execute();

//         // Ok(())

//         let tcp = std::net::TcpListener::bind(("127.0.0.1", 8088)).unwrap();

//         loop {
//             // Accept new sockets in a loop.
//             let (mut socket, addr) = match tcp.accept() {
//                 Ok(s) => s,
//                 Err(e) => {
//                     eprintln!("[WASM] > ERROR: {}", e);
//                     continue;
//                 }
//             };

//             println!("[WASM] > Accepted connection from {}", addr);

//             loop {
//                 // Create a buffer to read data into
//                 let mut buffer = vec![0; 1024];

//                 // Read data from the socket into the buffer
//                 match socket.read(&mut buffer) {
//                     Ok(size) => {
//                         // Write the same data back to the socket
//                         match socket.write(&buffer[..size]) {
//                             Ok(_) => {
//                             }
//                             Err(e) => {
//                                 eprintln!("[WASM] > ERROR writing back to {}: {}", addr, e);
//                                 break; // Exit inner loop if we encounter an error while writing
//                             }
//                         }
//                     }
//                     Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
//                         // No data available yet. Sleep for a short duration before retrying.
//                         sleep(Duration::from_millis(10));
//                     }
//                     Err(e) => {
//                         eprintln!("[WASM] > ERROR reading from {}: {}", addr, e);
//                         break; // Exit inner loop if we encounter an error while reading
//                     }
//                 }
//             }
//         }
//     });
// }

// #[test]
// fn test_dialer() -> Result<(), anyhow::Error> {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::INFO)
//         .init();

//     let conf = config::WATERConfig::init(String::from("./tests/test_wasm/proxy.wasm"), String::from("dial"), String::from("./tests/test_data/config.json"), 0)?;
//     let mut water_client = runtime::WATERClient::new(conf)?;
//     water_client.connect("", 0)?;

//     loop {
//         // keep reading from stdin and call read and write function from water_client.stream
//         let mut buf = String::new();
//         std::io::stdin().read_line(&mut buf)?;

//         water_client.write(buf.as_bytes())?;

//         let mut buf = vec![0; 1024];
//         water_client.read(&mut buf)?;

//         println!("read: {:?}", String::from_utf8_lossy(&buf));
//     }

//     Ok(())
// }
