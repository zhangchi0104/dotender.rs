use crate::types::{Config, Error};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    process,
};

pub fn parse_config<'a>(path: impl AsRef<Path>) -> Result<Config, Error<'a>> {
    let mut buf = String::new();
    fs::File::open(path)?.read_to_string(&mut buf)?;
    Ok(toml::from_str::<Config>(buf.as_str())?)
}

pub fn write_config(conf: &Config, path: impl AsRef<Path>) -> Result<(), Error> {
    let res = toml::to_string(conf)?;
    Ok(fs::File::open(path)?.write_all(res.as_bytes())?)
}

pub fn parse_hook(value: &str) -> Result<process::Command, Error> {
    let mut args = value.split(' ');
    let exec = args.next().ok_or(Error::HookParsingError(value))?;
    let mut cmd = process::Command::new(exec);
    cmd.args(args);
    Ok(cmd)
}
