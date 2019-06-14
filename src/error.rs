
use std::error::Error;

pub type Result<T> = std::result::Result<T, BenchError>;

#[derive(Debug)]
pub enum BenchError {
    // (command, return code)
    CommandFailed(String, i32),
    // (ip, command, return code)
    CommandFailedOnRemote(String, String, i32),
    // (command)
    NoSuchCommand(String),
    // (ip, command)
    NoSuchCommandOnRemote(String, String),
    // (command)
    CommandKilledBySingal(String),
    // (path)
    FileNotFound(String),

    // Wrapper
    ParseUtf8Error(std::string::FromUtf8Error),
    PraseIntError(std::num::ParseIntError),
    ParseTomlError(toml::de::Error),
    IoError(std::io::Error),
    // (message)
    Message(String)
    // CommandFails(i32),
    // Throw(String)
}

impl From<std::string::FromUtf8Error> for BenchError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        BenchError::ParseUtf8Error(error)
    }
}

impl From<toml::de::Error> for BenchError {
    fn from(error: toml::de::Error) -> Self {
        BenchError::ParseTomlError(error)
    }
}

impl From<std::num::ParseIntError> for BenchError {
    fn from(error: std::num::ParseIntError) -> Self {
        BenchError::PraseIntError(error)
    }
}

impl From<std::io::Error> for BenchError {
    fn from(error: std::io::Error) -> Self {
        BenchError::IoError(error)
    }
}

impl BenchError {
    pub fn as_remote_if_possible(self, ip: &str) -> Self {
        match self {
            BenchError::NoSuchCommand(cmd) =>
                BenchError::NoSuchCommandOnRemote(ip.to_owned(), cmd),
            BenchError::CommandFailed(cmd, code) =>
                BenchError::CommandFailedOnRemote(ip.to_owned(), cmd, code),
            other => other
        }
    }
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for BenchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BenchError::ParseUtf8Error(e) => Some(e),
            BenchError::ParseTomlError(e) => Some(e),
            BenchError::IoError(e) => Some(e),
            _ => None,
        }
    }
}