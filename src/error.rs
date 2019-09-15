
use std::error::Error;

pub type Result<T> = std::result::Result<T, BenchError>;

#[derive(Debug)]
pub enum BenchError {
    // (command, return code, stderr)
    CommandFailed(String, i32, String),
    // (ip, command, return code, stderr)
    CommandFailedOnRemote(String, String, i32, String),
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
    ParseIntError(std::num::ParseIntError),
    ParseFloatError(std::num::ParseFloatError),
    ParseBoolError(std::str::ParseBoolError),
    ParseTomlError(toml::de::Error),
    ParseJsonError(serde_json::error::Error),
    ParesPropertiesError(java_properties::PropertiesError),
    IoError(std::io::Error),
    CsvError(csv::Error),
    // PoisonError(std::sync::PoisonError),

    // (message)
    Message(String)
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
        BenchError::ParseIntError(error)
    }
}

impl From<std::num::ParseFloatError> for BenchError {
    fn from(error: std::num::ParseFloatError) -> Self {
        BenchError::ParseFloatError(error)
    }
}

impl From<std::str::ParseBoolError> for BenchError {
    fn from(error: std::str::ParseBoolError) -> Self {
        BenchError::ParseBoolError(error)
    }
}

impl From<serde_json::error::Error> for BenchError {
    fn from(error: serde_json::error::Error) -> Self {
        BenchError::ParseJsonError(error)
    }
}

impl From<java_properties::PropertiesError> for BenchError {
    fn from(error: java_properties::PropertiesError) -> Self {
        BenchError::ParesPropertiesError(error)
    }
}

impl From<std::io::Error> for BenchError {
    fn from(error: std::io::Error) -> Self {
        BenchError::IoError(error)
    }
}

impl<T> From<std::sync::PoisonError<T>> for BenchError {
    fn from(error: std::sync::PoisonError<T>) -> Self {
        BenchError::Message(format!("{:?}", error))
    }
}

impl From<csv::Error> for BenchError {
    fn from(error: csv::Error) -> Self {
        BenchError::CsvError(error)
    }
}

impl BenchError {
    pub fn as_remote_if_possible(self, ip: &str) -> Self {
        match self {
            BenchError::NoSuchCommand(cmd) =>
                BenchError::NoSuchCommandOnRemote(ip.to_owned(), cmd),
            BenchError::CommandFailed(cmd, code, stderr) =>
                BenchError::CommandFailedOnRemote(ip.to_owned(), cmd,
                    code, stderr),
            other => other
        }
    }
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BenchError::CommandFailed(cmd, code, stderr) => write!(f,
                "command '{}' fails with return code {}.\nError message: {}",
                cmd, code, stderr),
            BenchError::FileNotFound(path) => write!(f,
                "file not found: '{}'", path),
            BenchError::Message(s) => write!(f, "{}", s),
            e => write!(f, "{:?}", e)
        }
    }
}

impl Error for BenchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BenchError::ParseUtf8Error(e) => Some(e),
            BenchError::ParseIntError(e) => Some(e),
            BenchError::ParseFloatError(e) => Some(e),
            BenchError::ParseBoolError(e) => Some(e),
            BenchError::ParseTomlError(e) => Some(e),
            BenchError::ParseJsonError(e) => Some(e),
            BenchError::ParesPropertiesError(e) => Some(e),
            BenchError::IoError(e) => Some(e),
            _ => None,
        }
    }
}