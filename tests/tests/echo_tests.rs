use water::*;

use std::{
    fs::File,
    io::{Read, Write},
    net::TcpListener,
};

use tempfile::tempdir;

#[test]
fn test_echo() -> Result<(), Box<dyn std::error::Error>> {
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
        String::from("./test_wasm/echo_client.wasm"),
        String::from("_init"),
        String::from(file_path.to_string_lossy()),
        config::WaterBinType::Dial,
        true,
    )
    .unwrap();

    let mut water_client = runtime::WATERClient::new(conf).unwrap();
    water_client.connect("127.0.0.1", 8080).unwrap();
    water_client.write(test_message).unwrap();

    let mut buf = vec![0; 32];
    let res = water_client.read(&mut buf);
    assert!(res.is_ok());
    assert_eq!(res.unwrap() as usize, test_message.len());

    drop(file);
    dir.close()?;
    handle.join().unwrap();
    Ok(())
}
