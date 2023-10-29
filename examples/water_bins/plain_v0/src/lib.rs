#![no_main]

mod common;
mod error;
mod v0plus;

use lazy_static::lazy_static;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt},
};
use v0plus::ConnPair;

const READ_BUFFER_SIZE: usize = 1024; // 1KB is shorter than common MTU but longer than common TCP MSS

lazy_static! {
    static ref ROLE: Arc<Mutex<common::Role>> = Arc::new(Mutex::new(common::Role::Unknown));
    static ref DIALER: Arc<Mutex<v0plus::Dialer>> = Arc::new(Mutex::new(v0plus::Dialer::new()));
    static ref LISTENER: Arc<Mutex<v0plus::Listener>> =
        Arc::new(Mutex::new(v0plus::Listener::new()));
    static ref RELAY: Arc<Mutex<v0plus::Relay>> = Arc::new(Mutex::new(v0plus::Relay::new()));
    static ref CANCEL: Arc<Mutex<common::AsyncFdConn>> =
        Arc::new(Mutex::new(common::AsyncFdConn::new()));
}

#[export_name = "_water_v0"]
pub static VERSION: i32 = v0plus::VERSION;

// version-independent API
#[export_name = "_water_init"]
pub fn _init() -> i32 {
    // do all the initializing work here AND pull config from host
    sleep(Duration::from_millis(10)); // sleep for 10ms
    error::Error::None.i32()
}

// V0 API
#[export_name = "_water_dial"]
pub fn _dial(caller_conn_fd: i32) -> i32 {
    println!("Dialer: dialing...");

    // check ROLE, if set, return -1
    let mut role = ROLE.lock().unwrap();
    if *role != common::Role::Unknown {
        println!("_dial: role is already set to {:?}", *role);
        return error::Error::DoubleInit.i32();
    }

    // set ROLE to Dialer
    *role = common::Role::Dialer;

    let mut dialer = DIALER.lock().unwrap();
    match dialer.dial(caller_conn_fd) {
        Ok(remote_conn_fd) => {
            println!("Dialer: dial succeeded");
            remote_conn_fd
        }
        Err(e) => {
            println!("Dialer: dial failed: {}", e);
            error::Error::Unknown.i32()
        }
    }
}

// V0 API
#[export_name = "_water_accept"]
pub fn _accept(caller_conn_fd: i32) -> i32 {
    // check ROLE, if set, return -1
    let mut role = ROLE.lock().unwrap();
    if *role != common::Role::Unknown {
        println!("_accept: role is already set to {:?}", *role);
        return error::Error::DoubleInit.i32();
    }

    // set ROLE to Listener
    *role = common::Role::Listener;

    let mut listener = LISTENER.lock().unwrap();
    match listener.accept(caller_conn_fd) {
        Ok(source_conn_fd) => {
            println!("Listener: accept succeeded");
            source_conn_fd
        }
        Err(e) => {
            println!("Listener: listen failed: {}", e);
            error::Error::Unknown.i32()
        }
    }
}

// V0+ API
#[export_name = "_water_associate"]
pub fn _associate() -> i32 {
    // check ROLE, if set, return -1
    let mut role = ROLE.lock().unwrap();
    if *role != common::Role::Unknown {
        println!("_accept: role is already set to {:?}", *role);
        return error::Error::DoubleInit.i32();
    }

    // set ROLE to Relay
    *role = common::Role::Relay;

    let mut relay = RELAY.lock().unwrap();

    match relay.associate() {
        Ok(_) => {
            println!("Relay: associate succeeded");
            error::Error::None.i32()
        }
        Err(e) => {
            println!("Relay: associate failed: {}", e);
            error::Error::Unknown.i32()
        }
    }
}

// V0+ API
#[export_name = "_water_cancel_with"]
pub fn _cancel_with(fd: i32) -> i32 {
    // check ROLE, if not set, return -1
    let role = ROLE.lock().unwrap();
    if *role == common::Role::Unknown {
        println!("_cancel_with: role is not set");
        return error::Error::NotInitialized.i32();
    }

    // check CANCEL, if set, return -1
    let mut cancel = CANCEL.lock().unwrap();
    // set CANCEL
    match cancel.wrap(fd) {
        Ok(_) => {
            println!("_cancel_with: cancel set to {}", fd);
            error::Error::None.i32()
        }
        Err(e) => {
            println!("_cancel_with: cancel set failed: {}", e);
            error::Error::Unknown.i32()
        }
    }
}

/// WASM Entry point here
#[export_name = "_water_worker"]
pub fn _worker() -> i32 {
    // borrow CANCEL as &mut AsyncFdConn
    let mut cancel = CANCEL.lock().unwrap();
    let cancel = cancel.deref_mut();

    // check role
    let role = ROLE.lock().unwrap();
    match *role {
        common::Role::Dialer => {
            let mut dialer = DIALER.lock().unwrap();
            let conn_pair = dialer.conn_pair().expect("Dialer: conn_pair is None");
            let caller = conn_pair.0;
            let remote = conn_pair.1;
            match bidi_worker(remote, caller, cancel) {
                Ok(_) => {
                    dialer.close();
                    error::Error::None.i32()
                }
                Err(e) => {
                    println!("Dialer: bidi_worker failed: {}", e);
                    dialer.close();
                    error::Error::FailedIO.i32()
                }
            }
        }
        common::Role::Listener => {
            let mut listener = LISTENER.lock().unwrap();
            let conn_pair = listener.conn_pair().expect("Listener: conn_pair is None");
            let caller = conn_pair.0;
            let source = conn_pair.1;
            match bidi_worker(source, caller, cancel) {
                Ok(_) => {
                    listener.close();
                    error::Error::None.i32()
                }
                Err(e) => {
                    println!("Listener: bidi_worker failed: {}", e);
                    listener.close();
                    error::Error::FailedIO.i32()
                }
            }
        }
        common::Role::Relay => {
            let mut relay = RELAY.lock().unwrap();
            let conn_pair = relay.conn_pair().expect("Relay: conn_pair is None");
            let source = conn_pair.0;
            let remote = conn_pair.1;
            match bidi_worker(remote, source, cancel) {
                Ok(_) => {
                    relay.close();
                    error::Error::None.i32()
                }
                Err(e) => {
                    println!("Relay: bidi_worker failed: {}", e);
                    relay.close();
                    error::Error::FailedIO.i32()
                }
            }
        }
        _ => {
            println!("_worker: role is not set");
            error::Error::NotInitialized.i32()
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn bidi_worker(
    dst: &mut common::AsyncFdConn,
    src: &mut common::AsyncFdConn,
    cancel: &mut common::AsyncFdConn,
) -> std::io::Result<()> {
    // upgrade to AsyncFdConn
    dst.tokio_upgrade().expect("dst upgrade failed");
    src.tokio_upgrade().expect("src upgrade failed");
    cancel.tokio_upgrade().expect("cancel upgrade failed");

    let dst: &mut tokio::net::TcpStream = dst.stream().expect("dst stream is None");
    let src: &mut tokio::net::TcpStream = src.stream().expect("src stream is None");
    let cancel: &mut tokio::net::TcpStream = cancel.stream().expect("cancel stream is None");

    // dst.set_nodelay(true).expect("dst set_nodelay failed");
    // src.set_nodelay(true).expect("src set_nodelay failed");

    let mut dst_buf = vec![0; READ_BUFFER_SIZE];
    let mut src_buf = vec![0; READ_BUFFER_SIZE];
    let mut cancel_buf = vec![0; 256];

    loop {
        tokio::select! {
            result = dst.read(&mut dst_buf) => {
                // println!("dst.read() result = {:?}", result);
                match result {
                    Ok(0) => break, // End of stream
                    Ok(n) => {
                        if let Err(e) = src.write_all(&dst_buf[0..n]).await {
                            println!("Error writing to src: {:?}", e);
                            return Err(e);
                        }
                    }
                    Err(e) => {
                        println!("Error reading from dst: {:?}", e);
                        return Err(e);
                    }
                }
            }

            result = src.read(&mut src_buf) => {
                // println!("src.read() result = {:?}", result);
                match result {
                    Ok(0) => break, // End of stream
                    Ok(n) => {
                        if let Err(e) = dst.write_all(&src_buf[0..n]).await {
                            println!("Error writing to dst: {:?}", e);
                            return Err(e);
                        }
                    }
                    Err(e) => {
                        println!("Error reading from src: {:?}", e);
                        return Err(e);
                    }
                }
            }

            result = cancel.read(&mut cancel_buf) => {
                println!("cancel.read() result = {:?}", result);
                // exit
                break;
            }
        }
    }

    Ok(())
}
