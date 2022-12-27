use crate::types::{Config, Error};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process,
};
pub const CONFIG_VERSION: u16 = 1;
pub fn parse_config<'a>(path: impl AsRef<Path>) -> Result<Config, Error<'a>> {
    let mut buf = String::new();
    fs::File::open(path)?.read_to_string(&mut buf)?;
    let config = toml::from_str::<Config>(buf.as_str())?;
    if config.version != CONFIG_VERSION {
        Err(Error::InvalidConfigVersion(config.version))
    } else {
        Ok(config)
    }
}

pub fn write_config(conf: &Config, path: impl AsRef<Path>) -> Result<(), Error> {
    let res = toml::to_string(conf)?;
    Ok(fs::File::open(path)?.write_all(res.as_bytes())?)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn parse_hook(value: &str) -> Result<process::Command, Error> {
    let shell = match std::env::var("SHELL") {
        Ok(val) => val,
        Err(_) => String::from("/bin/sh"),
    };
    let mut cmd = process::Command::new(shell);
    let args = vec!["-c", value];
    cmd.args(args);
    Ok(cmd)
}
