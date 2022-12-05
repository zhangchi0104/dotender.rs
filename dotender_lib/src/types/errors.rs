use crate::config::CONFIG_VERSION;
use std::{env::VarError, fmt::Display, io::ErrorKind};
#[derive(Debug, PartialEq, Eq)]
pub struct HookExecutionError<'a> {
    status: Option<i32>,
    command: &'a str,
    stderr: String,
}

impl<'a> HookExecutionError<'a> {
    pub fn new(status: Option<i32>, command: &'a str, stderr: String) -> Self {
        Self {
            status,
            command,
            stderr,
        }
    }
}

impl Display for HookExecutionError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self.status {
            Some(v) => format!("exited with code {v}"),
            None => "terminated by signal".to_string(),
        };

        write!(
            f,
            "Hook '{}' exited with code {:?}, stderr: '{}'",
            self.command, status, self.stderr
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error<'error> {
    ParseError(toml::de::Error),
    SerializeError(toml::ser::Error),
    IoError(ErrorKind),
    HookParsingError(&'error str),
    FileMappingError(&'error str),
    InvalidConfigVersion(u16),
    HookExecutionError(HookExecutionError<'error>),
    InvalidPath(&'error str),
    Unexpected(String),
}

impl<'a> From<std::io::Error> for Error<'a> {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.kind())
    }
}

impl<'a> From<toml::de::Error> for Error<'a> {
    fn from(err: toml::de::Error) -> Self {
        Error::ParseError(err)
    }
}

impl<'a> From<toml::ser::Error> for Error<'a> {
    fn from(err: toml::ser::Error) -> Self {
        Error::SerializeError(err)
    }
}

impl<'err> From<VarError> for Error<'err> {
    fn from(error: VarError) -> Self {
        Error::Unexpected(format!("{error}"))
    }
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(e) => write!(f, "{e}"),
            Error::IoError(e) => write!(f, "{e}"),
            Error::SerializeError(e) => write!(f, "{e}"),
            Error::HookParsingError(cmd) => write!(f, "Command: '{cmd}' is invlaid"),
            Error::HookExecutionError(e) => write!(f, "{e}"),
            Error::FileMappingError(mapping) => write!(f, "invalid mapping '{mapping}'"),
            Error::InvalidConfigVersion(ver) => {
                write!(f, "Expected config version = {CONFIG_VERSION}, got {ver}")
            }
            Error::InvalidPath(path) => writeln!(f, "Invalid path: {path}"),
            Error::Unexpected(e) => writeln!(f, "{e}"),
        }
    }
}
