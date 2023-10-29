use crate::{common::*, error};

pub const VERSION: i32 = 0x00000000; // v0plus share the same version number with v0

pub struct Dialer {
    caller_conn: AsyncFdConn,
    remote_conn: AsyncFdConn,
}

pub struct Listener {
    caller_conn: AsyncFdConn,
    source_conn: AsyncFdConn,
}
pub struct Relay {
    source_conn: AsyncFdConn,
    remote_conn: AsyncFdConn,
}

impl Dialer {
    pub fn new() -> Self {
        Dialer {
            caller_conn: AsyncFdConn::new(),
            remote_conn: AsyncFdConn::new(),
        }
    }

    pub fn dial(&mut self, caller_conn_fd: i32) -> Result<i32, String> {
        // check if caller_conn_fd is valid
        if caller_conn_fd < 0 {
            return Err("invalid caller_conn_fd".to_string());
        }
        match self.caller_conn.wrap(caller_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // call external dial() to get remote_conn_fd
        let remote_conn_fd = unsafe { host_dial() };
        if remote_conn_fd < 0 {
            return Err("dial failed".to_string());
        }
        match self.remote_conn.wrap(remote_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // return remote_conn_fd
        Ok(remote_conn_fd)
    }

    // // borrow self.caller_conn
    // pub fn caller(&mut self) -> Option<&mut TcpStream> {
    //     self.caller_conn.stream()
    // }

    // // borrow self.remote_conn
    // pub fn remote(&mut self) -> Option<&mut TcpStream> {
    //     self.remote_conn.stream()
    // }

    pub fn close(&mut self) {
        self.caller_conn.close();
        self.remote_conn.close();
        unsafe { host_defer() };
    }
}

impl Listener {
    pub fn new() -> Self {
        Listener {
            caller_conn: AsyncFdConn::new(),
            source_conn: AsyncFdConn::new(),
        }
    }

    pub fn accept(&mut self, caller_conn_fd: i32) -> Result<i32, String> {
        // check if caller_conn_fd is valid
        if caller_conn_fd < 0 {
            return Err("Listener: invalid caller_conn_fd".to_string());
        }

        match self.caller_conn.wrap(caller_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // call external accept() to get source_conn_fd
        let source_conn_fd = unsafe { host_accept() };
        if source_conn_fd < 0 {
            return Err("Listener: accept failed".to_string());
        }

        match self.source_conn.wrap(source_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // return source_conn_fd
        Ok(source_conn_fd)
    }

    // // borrow self.caller_conn
    // pub fn caller(&mut self) -> Option<&mut TcpStream> {
    //     self.caller_conn.stream()
    // }

    // // borrow self.source_conn
    // pub fn source(&mut self) -> Option<&mut TcpStream> {
    //     self.source_conn.stream()
    // }

    pub fn close(&mut self) {
        self.caller_conn.close();
        self.source_conn.close();
        unsafe { host_defer() };
    }
}

impl Relay {
    pub fn new() -> Self {
        Relay {
            source_conn: AsyncFdConn::new(),
            remote_conn: AsyncFdConn::new(),
        }
    }

    pub fn associate(&mut self) -> Result<i32, String> {
        // call external accept() to get source_conn_fd
        let source_conn_fd = unsafe { host_accept() };
        if source_conn_fd < 0 {
            return Err("Relay: accept failed".to_string());
        }

        match self.source_conn.wrap(source_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // call external dial() to get remote_conn_fd
        let remote_conn_fd = unsafe { host_dial() };
        if remote_conn_fd < 0 {
            return Err("Relay: dial failed".to_string());
        }
        match self.remote_conn.wrap(remote_conn_fd) {
            Ok(_) => {}
            Err(e) => {
                return Err(e);
            }
        }

        // return remote_conn_fd
        Ok(error::Error::None.i32())
    }

    // // borrow self.source_conn
    // pub fn source(&mut self) -> Option<&mut TcpStream> {
    //     self.source_conn.stream()
    // }

    // // borrow self.remote_conn
    // pub fn remote(&mut self) -> Option<&mut TcpStream> {
    //     self.remote_conn.stream()
    // }

    pub fn close(&mut self) {
        self.source_conn.close();
        self.remote_conn.close();
        unsafe { host_defer() };
    }
}

pub trait ConnPair {
    fn conn_pair(&mut self) -> Option<(&mut AsyncFdConn, &mut AsyncFdConn)>;
}

impl ConnPair for Dialer {
    fn conn_pair(&mut self) -> Option<(&mut AsyncFdConn, &mut AsyncFdConn)> {
        Some((&mut self.caller_conn, &mut self.remote_conn))
    }
}

impl ConnPair for Listener {
    fn conn_pair(&mut self) -> Option<(&mut AsyncFdConn, &mut AsyncFdConn)> {
        Some((&mut self.caller_conn, &mut self.source_conn))
    }
}

impl ConnPair for Relay {
    fn conn_pair(&mut self) -> Option<(&mut AsyncFdConn, &mut AsyncFdConn)> {
        Some((&mut self.source_conn, &mut self.remote_conn))
    }
}
