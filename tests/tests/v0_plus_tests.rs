//! This is the test file for testing the plain.wasm which is a v0_plus WATM module that has been tested with the Go engine.
//!
//! Tests here are showing that the same WATM module can be used interchangeably in both the Rust and Go engine.

#![allow(dead_code)]

use water::{runtime::client::WATERClient, *};

use tracing::Level;

use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    thread::JoinHandle,
    vec,
};

use tempfile::tempdir;

/// Testing the Dialer mode
#[test]
fn test_cross_lang_wasm_dialer() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cfg_str = r#"
	{
		"remote_address": "127.0.0.1",
		"remote_port": 8080,
		"local_address": "127.0.0.1",
		"local_port": 8088
	}
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    let test_message = b"hello";
    let handle = std::thread::spawn(|| {
        let listener = TcpListener::bind(("127.0.0.1", 8080)).unwrap();
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
        // https://github.com/refraction-networking/water/tree/master/examples/v0/plain
        //
        // Source code of plain.wasm:
        // https://github.com/refraction-networking/water-rs/tree/main/examples/water_bins/plain_v0
        //
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Dial,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();
    water_client.connect().unwrap();
    water_client.cancel_with().unwrap();

    let handle_water = water_client.run_worker().unwrap();
    water_client.write(test_message).unwrap();

    let mut buf = vec![0; 32];
    let res = water_client.read(&mut buf);
    assert!(res.is_ok());
    assert_eq!(res.unwrap() as usize, test_message.len());

    water_client.cancel().unwrap();

    drop(file);
    dir.close()?;
    handle.join().unwrap();
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

/// Testing the Listener mode
#[test]
fn test_cross_lang_wasm_listener() -> Result<(), Box<dyn std::error::Error>> {
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

    let test_message = b"hello";

    let conf = config::WATERConfig::init(
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Listen,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();
    water_client.listen().unwrap();

    let handle = std::thread::spawn(|| {
        // give some time let the listener start to accept
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut stream = TcpStream::connect(("127.0.0.1", 8082)).unwrap();
        let res = stream.write(test_message);

        assert!(res.is_ok());
        let write_bytes = res.unwrap();

        assert_eq!(write_bytes, test_message.len());
    });

    water_client.accept().unwrap();

    water_client.cancel_with().unwrap();

    let handle_water = water_client.run_worker().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut buf = vec![0; 32];
    let res = water_client.read(&mut buf);
    assert!(res.is_ok());
    assert_eq!(res.unwrap() as usize, test_message.len());

    water_client.cancel().unwrap();

    drop(file);
    dir.close()?;
    handle.join().unwrap();
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

/// Testing the listener mode but with multiple incoming connections
#[test]
fn test_cross_lang_wasm_multi_listener() -> Result<(), Box<dyn std::error::Error>> {
    let cfg_str = r#"
	{
		"remote_address": "127.0.0.1",
		"remote_port": 8088,
		"local_address": "127.0.0.1",
		"local_port": 10088
	}
	"#;
    // Create a directory inside of `std::env::temp_dir()`.
    let dir = tempdir()?;
    let file_path = dir.path().join("temp-config.txt");
    let mut file = File::create(&file_path)?;
    writeln!(file, "{}", cfg_str)?;

    let conf = config::WATERConfig::init(
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Listen,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();
    water_client.listen().unwrap();

    let test_message: &'static [u8] = b"hello";

    let mut water_handles: Vec<JoinHandle<()>> = Vec::new();

    // creating two connections to the listener
    for _i in 0..2 {
        // make a connect to the listener in a separate thread
        std::thread::spawn(|| {
            let mut stream = TcpStream::connect(("127.0.0.1", 10088)).unwrap();
            let res = stream.write(test_message);

            assert!(res.is_ok());
            let write_bytes = res.unwrap();

            assert_eq!(write_bytes, test_message.len());
        });

        water_client.accept().unwrap();

        let new_water = water_client.keep_listen().unwrap();

        water_handles.push(std::thread::spawn(|| {
            handle_connection(water_client, test_message).unwrap();
        }));

        water_client = new_water;
    }

    for handle in water_handles {
        handle.join().unwrap();
    }

    drop(file);
    dir.close()?;

    Ok(())
}

fn handle_connection(
    mut water_client: WATERClient,
    test_message: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    water_client.cancel_with().unwrap();

    let handle = water_client.run_worker().unwrap();

    let mut buf = vec![0; 32];
    let res = water_client.read(&mut buf);
    assert!(res.is_ok());
    assert_eq!(res.unwrap() as usize, test_message.len());

    water_client.cancel().unwrap();

    let _ = handle.join().unwrap();

    Ok(())
}
