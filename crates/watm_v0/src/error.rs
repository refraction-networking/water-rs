//! Error codes for the WATM module

/// Error is a enum in i32
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error {
    None = 0,
    /// general error
    Unknown = -1,
    /// invalid argument supplied to func call
    InvalidArgument = -2,
    /// config file provided is invalid
    InvalidConfig = -3,
    /// invalid file descriptor provided
    InvalidFd = -4,
    /// invalid function called
    InvalidFunction = -5,
    /// initializing twice
    DoubleInit = -6,
    /// Failing an I/O operation
    FailedIO = -7,
    /// not initialized
    NotInitialized = -8,
}

impl Error {
    pub fn i32(&self) -> i32 {
        *self as i32
    }
}
