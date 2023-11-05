use water::*;

use tracing::Level;

use std::{
    fs::File,
    io::{Error, ErrorKind, Read, Write},
    net::TcpListener,
};

use tempfile::tempdir;

#[test]
fn test_cross_lan_wasm() -> Result<(), Box<dyn std::error::Error>> {
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
        // let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
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
        // https://github.com/gaukas/water/tree/master/examples/v0/plain
        //
        // More details for the implementation of plain.wasm check this PR:
        // https://github.com/erikziyunchi/water-rs/pull/10
        //
        String::from("./test_wasm/plain.wasm"),
        String::from("_water_worker"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Dial,
        true,
    )
    .unwrap();

    let mut water_client = runtime::client::WATERClient::new(conf).unwrap();
    water_client.connect("127.0.0.1", 8080).unwrap();
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
