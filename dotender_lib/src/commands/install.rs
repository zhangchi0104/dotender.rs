use std::{fs::create_dir_all, process};

use crate::{
    config::parse_hook,
    types::{
        self,
        args::{self, InstallArgs},
        config::ConfigItem,
        errors::HookExecutionError,
        Error,
    },
    ui::{
        install::{InstallStep, InstallUI},
        StatefulUI,
    },
    utils::split_mapping,
};
use indicatif::{MultiProgress, ProgressBar};
use rayon::prelude::*;
pub struct Install {
    args: args::InstallArgs,
}

impl Install {
    fn run_hooks<'hooks>(
        &self,
        hooks: &'hooks [String],
        mut update_with: impl FnMut(&'hooks str),
    ) -> Result<(), Error<'hooks>> {
        hooks.iter().try_for_each(|hook| {
            let mut cmd = parse_hook(hook)?;
            update_with(hook);

            if self.args.dry_run {
                return Ok(());
            }

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

    fn do_install<'config>(
        &self,
        item: &'config ConfigItem,
        ui: &mut InstallUI<'config>,
    ) -> Result<(), Error<'config>> {
        if let Some(hooks) = &item.before {
            self.run_hooks(hooks, |hook| ui.update(InstallStep::PreInstallHook(hook)))?
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
            self.run_hooks(hooks, |hook| ui.update(InstallStep::PostInstallHook(hook)))?
        }
        ui.done_with_msg("Done");
        Ok(())
    }

    #[cfg(unix)]
    fn create_link<'a>(&self, origin: &'a str, dst: &'a str) -> Result<(), Error<'a>> {
        use std::os::unix::fs;
        if self.args.dry_run {
            return Ok(());
        }
        use crate::utils::absolute_path;
        let dst_path = absolute_path(dst)?;
        let origin_path = absolute_path(origin)?;
        if self.args.parent {
            dst_path.parent().map_or(Ok(()), create_dir_all)?;
        }
        Ok(fs::symlink(origin_path, dst_path)?)
    }

    #[cfg(windows)]
    fn create_link<'a>(&self, src: &'a str, dst: &'a str) -> Result<(), Error<'a>> {
        todo!()
    }
}

impl types::Command for Install {
    fn run(&mut self, cfg: &mut crate::types::Config) {
        let m = MultiProgress::new();
        cfg.dotfiles.par_iter().for_each(|(k, v)| {
            // println!("{k}");
            let pb = m.add(ProgressBar::new_spinner());
            pb.set_prefix(k.to_string());
            let mut ui = InstallUI::from(pb);
            let res = self.do_install(v, &mut ui);
            match res {
                Ok(_) => ui.done_with_msg("Done"),
                Err(e) => ui.pbar.set_message(format!("Failed: {e}")),
            }
        });
    }
}

impl From<InstallArgs> for Install {
    fn from(args: InstallArgs) -> Self {
        Self { args }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::types::{args::InstallArgs, Error};

    use super::Install;

    fn install_fixture(args: Option<InstallArgs>) -> Install {
        let install_args = match args {
            Some(args) => args,
            None => InstallArgs::default(),
        };
        Install { args: install_args }
    }

    #[test]
    fn test_create_link_success() -> Result<(), Error<'static>> {
        let install = install_fixture(None);
        let result = install.create_link("../test_env/foo.txt", "../test_env/foo_link.txt");
        let _ = fs::remove_file("../test_env/foo_link.txt");
        result
    }
}
