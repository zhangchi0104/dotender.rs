use std::{fs::create_dir_all, path::PathBuf, process};

use crate::{
    config::parse_hook,
    types::{self, args, config::ConfigItem, errors::HookExecutionError, Error},
    utils::split_mapping,
};
use rayon::prelude::*;
pub struct Install {
    args: args::InstallArgs,
}

impl Install {
    fn run_hooks(hooks: &[String]) -> Result<(), Error> {
        hooks.iter().try_for_each(|hook| {
            let mut cmd = parse_hook(hook)?;
            let child = cmd.stderr(process::Stdio::piped()).output()?;
            let status = child.status;
            let stderr = String::from_utf8_lossy(&child.stderr);
            if !status.success() {
                let err =
                    HookExecutionError::new(status.code(), hook.as_str(), String::from(stderr));
                Err(Error::HookExecutionError(err))
            } else {
                Ok(())
            }
        })
    }

    fn do_install<'a>(&self, item: &'a ConfigItem) -> Result<(), Error<'a>> {
        if let Some(hooks) = &item.before {
            Self::run_hooks(hooks)?
        }
        // parse links
        item.mappings
            .iter()
            .map(|s| split_mapping(s.as_str()))
            .try_for_each(|mapping| {
                let (src, dst) = mapping?;
                self.create_link(src, dst)?;
                Ok::<(), Error>(())
            })?;

        if let Some(hooks) = &item.after {
            Self::run_hooks(hooks)?
        }

        Ok(())
    }

    #[cfg(unix)]
    fn create_link<'a>(&self, src: &'a str, dst: &'a str) -> Result<(), Error<'a>> {
        use std::os::unix::fs;
        let dst_path = PathBuf::from(dst);
        if self.args.parent {
            dst_path.parent().map_or(Ok(()), create_dir_all)?;
        }
        Ok(fs::symlink(src, dst)?)
    }

    #[cfg(windows)]
    fn create_link<'a>(&self, src: &'a str, dst: &'a str) -> Result<(), Error<'a>> {
        todo!()
    }
}

impl types::Command for Install {
    fn run<'a>(&mut self, cfg: &'a mut crate::types::Config) -> Result<(), Error<'a>> {
        cfg.dotfiles
            .par_iter()
            .map(|(_, v)| self.do_install(v))
            .collect::<Result<_, _>>()?;
        Ok(())
    }
}
