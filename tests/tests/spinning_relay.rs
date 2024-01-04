#![allow(dead_code)]

use water::*;

use tracing::Level;

use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
};

use tempfile::tempdir;

#[test]
fn test_cross_lang_wasm_relay() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cfg_str = r#"
	{
		"remote_address": "127.0.0.1",
		"remote_port": 8088,
		"local_address": "127.0.0.1",
		"local_port": 8080
	}
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    let test_message = b"hello";

    // starting the listener in another thread it to relay to
    let handle_remote = std::thread::spawn(|| {
        let listener = TcpListener::bind(("127.0.0.1", 8088)).unwrap();
        let (mut socket, _) = listener.accept().unwrap();
        let mut buf = [0; 1024];
        let res = socket.read(&mut buf);

        assert!(res.is_ok());
        let read_bytes = res.unwrap();
        assert_eq!(read_bytes, test_message.len());

        let res = socket.write(&buf[..read_bytes]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), test_message.len());
    });

    let conf = config::WATERConfig::init(
        // plain.wasm is in v0 and fully compatible with the Go engine
        // More details for the Go-side of running plain.wasm check here:
        // https://github.com/gaukas/water/tree/master/examples/v0/plain
        //
        // More details for the implementation of plain.wasm check this PR:
        // https://github.com/erikziyunchi/water-rs/pull/10
        //
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Relay,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();

    water_client.relay().unwrap();

    // connects to the relay, and the relay will connect to the listener
    let handle_local = std::thread::spawn(|| {
        // give some time let the listener start to accept
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut stream = TcpStream::connect(("127.0.0.1", 8080)).unwrap();

        let res = stream.write(test_message);
        assert!(res.is_ok());
        let write_bytes = res.unwrap();
        assert_eq!(write_bytes, test_message.len());

        let mut buf = [0; 1024];
        let res = stream.read(&mut buf);
        assert!(res.is_ok());
        let read_bytes = res.unwrap();
        assert_eq!(read_bytes, test_message.len());
    });

    water_client.associate().unwrap();
    water_client.cancel_with().unwrap();

    let handle_water = water_client.run_worker().unwrap();

    // give it a second before cancel to let the connector check correct transfer
    std::thread::sleep(std::time::Duration::from_secs(2));

    water_client.cancel().unwrap();

    drop(file);
    dir.close()?;
    handle_remote.join().unwrap();
    handle_local.join().unwrap();
    match handle_water.join().unwrap() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Running _water_worker ERROR: {}", e);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Failed to join _water_worker thread",
            )));
        }
    };

    Ok(())
}

// #[test]
fn spin_cross_lang_wasm_relay() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cfg_str = r#"
	{
		"remote_address": "127.0.0.1",
		"remote_port": 8088,
		"local_address": "127.0.0.1",
		"local_port": 8082
	}
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    let conf = config::WATERConfig::init(
        // plain.wasm is in v0 and fully compatible with the Go engine
        // More details for the Go-side of running plain.wasm check here:
        // https://github.com/gaukas/water/tree/master/examples/v0/plain
        //
        // More details for the implementation of plain.wasm check this PR:
        // https://github.com/erikziyunchi/water-rs/pull/10
        //
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Relay,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();

    water_client.relay().unwrap();

    water_client.associate().unwrap();
    water_client.cancel_with().unwrap();

    let handle_water = water_client.run_worker().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(20));

    water_client.cancel().unwrap();

    drop(file);
    dir.close()?;
    match handle_water.join().unwrap() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Running _water_worker ERROR: {}", e);
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Failed to join _water_worker thread",
            )));
        }
    };

    Ok(())
}
