use std::{env, path::PathBuf};

use crate::types::Error;

pub fn split_mapping(mapping_raw: &str) -> Result<(&str, &str), Error> {
    mapping_raw
        .split_once(':')
        .ok_or(Error::FileMappingError(mapping_raw))
}
pub fn into_absolute_path(path: String) -> Option<PathBuf> {
    let expanded_user_path = if path.starts_with("~/") {
        let home = env::var("HOME").ok()? + "/";
        path.replacen("~/", home.as_str(), 1)
    } else {
        path
    };
    PathBuf::from(expanded_user_path).canonicalize().ok()
}

pub fn absolute_path(path: &str) -> Result<PathBuf, Error> {
    let mut res = PathBuf::new();
    let p = if path.starts_with("~/") {
        let home = env::var("HOME")? + "/";
        path.replacen("~/", home.as_str(), 1)
    } else if !path.starts_with('/') {
        let anchor = env::current_dir()?;
        res.push(anchor);
        path.to_string()
    } else {
        path.to_string()
    };

    p.split(std::path::MAIN_SEPARATOR).try_for_each(|part| {
        match part {
            "." => (),
            ".." => {
                if !res.pop() {
                    return Err(Error::InvalidPath(path));
                }
            }
            _ => res.push(part),
        }
        Ok(())
    })?;
    Ok(res)
}
