use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use std::ops::Deref;
use std::convert::{TryFrom, TryInto};

// ========= Definition for files shared between WASM & Host for creating connections ===========
// TODO: migrate these code to a src file later
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
/// Name assigned to a file descriptor
///
/// This is used to export the `FD_NAMES` environment variable,
/// which is a concatenation of all file descriptors names seperated by `:`.
///
/// See the [crate] documentation for examples.
pub struct FileName(String);

impl TryFrom<String> for FileName {
    type Error = &'static str;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if name.find(':').is_some() {
            Err("file name must not contain ':'")
        } else {
            Ok(Self(name))
        }
    }
}

impl TryFrom<&str> for FileName {
    type Error = <FileName as TryFrom<String>>::Error;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        String::from(name).try_into()
    }
}

impl Deref for FileName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FileName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        name.try_into().map_err(D::Error::custom)
    }
}

const fn default_tcp_port() -> u16 {
    80
}

const fn default_tls_port() -> u16 {
    443
}

fn default_addr() -> String {
    "::".into()
}

/// Parameters for a pre-opened file descriptor
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum File {
    // /// File descriptor of `/dev/null`
    // #[serde(rename = "null")]
    // Null(NullFile),

    // /// File descriptor of stdin
    // #[serde(rename = "stdin")]
    // Stdin(StdioFile),

    // /// File descriptor of stdout
    // #[serde(rename = "stdout")]
    // Stdout(StdioFile),

    // /// File descriptor of stderr
    // #[serde(rename = "stderr")]
    // Stderr(StdioFile),

    /// File descriptor of a listen socket
    #[serde(rename = "listen")]
    Listen(ListenFile),

    /// File descriptor of a stream socket
    #[serde(rename = "connect")]
    Connect(ConnectFile),
}

impl File {
    /// Get the name for a file descriptor
    pub fn name(&self) -> &str {
        match self {
            // Self::Null(NullFile { name }) => name.as_deref().unwrap_or("null"),
            // Self::Stdin(StdioFile { name }) => name.as_deref().unwrap_or("stdin"),
            // Self::Stdout(StdioFile { name }) => name.as_deref().unwrap_or("stdout"),
            // Self::Stderr(StdioFile { name }) => name.as_deref().unwrap_or("stderr"),
            Self::Listen(ListenFile::Tls { name, .. }) => name,
            Self::Listen(ListenFile::Tcp { name, .. }) => name,
            Self::Connect(ConnectFile::Tls { name, host, .. }) => name.as_deref().unwrap_or(host),
            Self::Connect(ConnectFile::Tcp { name, host, .. }) => name.as_deref().unwrap_or(host),
        }
    }
}

/// File descriptor of a listen socket
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "prot", deny_unknown_fields)]
pub enum ListenFile {
    /// TLS listen socket
    #[serde(rename = "tls")]
    Tls {
        /// Name assigned to the file descriptor
        name: FileName,

        /// Address to listen on
        #[serde(default = "default_addr")]
        addr: String,

        /// Port to listen on
        #[serde(default = "default_tls_port")]
        port: u16,
    },

    /// TCP listen socket
    #[serde(rename = "tcp")]
    Tcp {
        /// Name assigned to the file descriptor
        name: FileName,

        /// Address to listen on
        #[serde(default = "default_addr")]
        addr: String,

        /// Port to listen on
        #[serde(default = "default_tcp_port")]
        port: u16,
    },
}

/// File descriptor of a stream socket
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "prot", deny_unknown_fields)]
pub enum ConnectFile {
    /// TLS stream socket
    #[serde(rename = "tls")]
    Tls {
        /// Name assigned to the file descriptor
        name: Option<FileName>,

        /// Host address to connect to
        host: String,

        /// Port to connect to
        #[serde(default = "default_tls_port")]
        port: u16,
    },

    /// TCP stream socket
    #[serde(rename = "tcp")]
    Tcp {
        /// Name assigned to the file descriptor
        name: Option<FileName>,

        /// Host address to connect to
        host: String,

        /// Port to connect to
        #[serde(default = "default_tcp_port")]
        port: u16,
    },
}