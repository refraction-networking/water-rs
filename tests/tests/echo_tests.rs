
use water::*;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tempfile::tempdir;

use std::io::Write;
use std::fs::File;

#[tokio::test]
async fn test_echo() -> Result<(), Box<dyn std::error::Error>> {

	let cfg_str = r#"
	{
		"server": "127.0.0.1",
		"server_port": 8080,
	}
	"#;
	// Create a directory inside of `std::env::temp_dir()`.
	let dir = tempdir()?;
	let file_path = dir.path().join("my-temporary-note.txt");
	let mut file = File::create(&file_path)?;
	writeln!(file,"{}", cfg_str)?;

	let test_message = b"hello";
	tokio::spawn(async move {
		let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
		let (mut socket, _) = listener.accept().await.unwrap();
		let mut buf = [0; 1024];
		let res = socket.read(&mut buf).await;
		assert!(res.is_ok());
		assert_eq!(res.unwrap(), test_message.len());
		let res = socket.write(&buf).await;
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
	Ok(())
}