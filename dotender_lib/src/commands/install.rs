use std::{fs::create_dir_all, process};

use crate::ui::install::InstallStep::Link;
use crate::utils::find_invalid_keys;
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

#[derive(Debug)]
pub struct Install {
    pub(super) args: args::InstallArgs,
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

            if self.args.dry_run || self.args.skip_hooks {
                return Ok(());
            }

            let child = cmd.stderr(process::Stdio::piped()).output()?;
            let status = child.status;
            let stderr = String::from_utf8_lossy(&child.stderr);
            if !status.success() {
                let err =
                    HookExecutionError::new(status.code(), hook.as_ref(), String::from(stderr));
                Err(Error::HookExecutionError(err))
            } else {
                Ok(())
            }
        })
    }

    fn do_install<'config>(
        &self,
        item: &'config ConfigItem,
        ui: &mut impl StatefulUI<InstallStep<'config>>,
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
                ui.update(Link(src, dst));
                Ok::<(), Error>(())
            })?;

        if let Some(hooks) = &item.after {
            self.run_hooks(hooks, |hook| ui.update(InstallStep::PostInstallHook(hook)))?
        }
        ui.done_with_msg("Done");
        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    pub(crate) fn create_link<'a>(&self, origin: &'a str, dst: &'a str) -> Result<(), Error<'a>> {
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

        if self.args.force && dst_path.exists() {
            let _ = std::fs::remove_dir_all(&dst_path);
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
        // check if items are valid
        let invalid_key = find_invalid_keys(&self.args.items, &cfg.dotfiles);

        if let Some(k) = invalid_key {
            eprintln!("install failed: {}", Error::InvalidItem(k.as_str()))
        }

        // println!("{k}");
        self.args.items.par_iter().for_each(|k| {
            let pb = m.add(ProgressBar::new_spinner());
            let config = cfg.dotfiles.get(k).unwrap();
            pb.set_prefix(k.to_string());
            let mut ui = InstallUI::from(pb);
            let res = self.do_install(config, &mut ui);
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

    use crate::{
        types::{args::InstallArgs, config::ConfigItem},
        ui::{install::InstallStep, StatefulUI},
    };
    use std::{fs, io::Read, ops::Deref, path::PathBuf};
    struct TestUI;
    impl StatefulUI<InstallStep<'_>> for TestUI {
        fn update(&mut self, _new_state: InstallStep<'_>) {
            ()
        }

        fn done_with_msg(&mut self, _msg: &'static str) {
            ()
        }
    }

    use super::Install;

    fn install_fixture(args: Option<InstallArgs>, linkname: &str) -> InstallFixture {
        let install_args = match args {
            Some(args) => args,
            None => InstallArgs::default(),
        };
        InstallFixture {
            cmd: Install { args: install_args },
            link: linkname,
        }
    }

    #[derive(Debug)]
    struct InstallFixture<'fixture> {
        cmd: Install,
        link: &'fixture str,
    }

    impl Drop for InstallFixture<'_> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.link);
        }
    }

    impl Deref for InstallFixture<'_> {
        type Target = Install;

        fn deref(&self) -> &Self::Target {
            &self.cmd
        }
    }

    #[test]
    fn test_create_link_success() {
        let fixture = install_fixture(None, ".foo_link_success.txt");
        let result = fixture.create_link("./Cargo.toml", fixture.link);

        assert!(result.is_ok())
    }

    #[test]
    fn test_create_link_should_not_override() {
        let fixture = install_fixture(None, "./foo_link_override_fail.txt");
        let _ = fixture.create_link("./Cargo.toml", fixture.link);
        let result = fixture.create_link("./Cargo.toml", fixture.link);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_link_should_override_with_force() {
        let fixture = install_fixture(
            Some(InstallArgs {
                force: true,
                parent: false,
                dry_run: false,
                items: vec![String::from("zsh")],
                skip_hooks: false,
            }),
            "./foo_link_override_force.txt",
        );
        let res = fixture.create_link("./Cargo.toml", fixture.link);
        assert!(res.is_ok());
        let res = fixture.create_link("./Cargo.toml", fixture.link);
        assert!(res.is_ok());
    }

    #[test]
    fn test_run_hooks() {
        let fixture = install_fixture(None, "./run_hooks.txt");
        let hooks = vec![String::from("touch ./run_hooks.txt")];
        let res = fixture.run_hooks(hooks.as_slice(), |_| eprintln!("Done"));
        assert!(PathBuf::from("./run_hooks.txt").exists());
        assert!(res.is_ok());
    }

    #[test]
    fn test_run_multiple_hooks() {
        let fixture = install_fixture(None, "./multiple_hooks.txt");
        let hooks = vec![
            String::from("touch ./multiple_hooks.txt"),
            String::from("rm ./multiple_hooks.txt"),
        ];
        let res = fixture.run_hooks(hooks.as_slice(), |_| {
            let _ = 1 + 1;
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_hooks_extra_spaces() {
        let fixture = install_fixture(None, "./extra_space.txt");
        let hooks = vec![String::from("touch   ./extra_space.txt")];
        let res = fixture.run_hooks(hooks.as_slice(), |_| ());
        assert!(PathBuf::from("./extra_space.txt").exists());
        assert!(res.is_ok());
    }

    #[test]
    fn test_hooks_quotation_mark_spaces() {
        let fixture = install_fixture(None, "'sapce space'.txt");
        let hooks = vec![format!("touch {}", "\"'sapce space'\".txt")];
        let res = fixture.run_hooks(hooks.as_slice(), |_| ());
        assert!(res.is_ok());
        assert!(PathBuf::from(fixture.link).exists());
    }

    #[test]
    fn test_hooks_failed() {
        let fixture = install_fixture(None, "./hooks_failed.txt");
        let hooks = vec![String::from("invalid-cmd foo bar")];
        let res = fixture.run_hooks(hooks.as_slice(), |_| ());
        assert!(res.is_err());
    }

    #[test]
    fn test_hooks_with_pipe() {
        let fixture = install_fixture(None, "./hook_with_pipe.txt");
        let hooks = vec![
            String::from("echo test > ./hook_with_pipe.txt"),
            String::from("echo test2 >> ./hook_with_pipe.txt"),
        ];
        let _ = fixture.run_hooks(hooks.as_slice(), |_| ());
        let mut file = std::fs::File::open(fixture.link).unwrap();

        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "test\ntest2\n");
    }

    #[test]
    fn test_do_install() {
        let fixture = install_fixture(None, "./test_do_install.txt");
        let config = ConfigItem {
            mappings: vec![String::from("./Cargo.toml:./test_do_install.txt")],
            before: Some(vec![
                String::from("echo test_before > /dev/null"),
                String::from("echo test_before1 > /dev/null"),
            ]),
            after: Some(vec![
                String::from("echo test_after > /dev/null"),
                String::from("echo test_after1 > /dev/null"),
            ]),
        };
        let mut ui = TestUI {};
        let result = fixture.do_install(&config, &mut ui);
        assert!(PathBuf::from(fixture.link).exists());
        assert!(result.is_ok())
    }
    #[test]
    fn test_do_install_invalid_pre_hooks() {
        let fixture = install_fixture(None, "./test_do_install_pre_hooks.txt");
        let config = ConfigItem {
            mappings: vec![format!("./Cargo.toml:{}", fixture.link)],
            before: Some(vec![
                String::from("invalid_cmd test_before > /dev/null"),
                String::from("echo test_before1 > /dev/null"),
            ]),
            after: None,
        };
        let mut ui = TestUI {};
        let result = fixture.do_install(&config, &mut ui);
        assert!(!PathBuf::from(fixture.link).exists());
        assert!(result.is_err())
    }

    #[test]
    fn test_do_install_invalid_post_hooks() {
        let fixture = install_fixture(None, "./test_do_install_post_hooks.txt");
        let config = ConfigItem {
            mappings: vec![format!("./Cargo.toml:{}", fixture.link)],
            before: None,
            after: Some(vec![
                String::from("invalid_cmd test_before > /dev/null"),
                String::from("echo test_before1 > /dev/null"),
            ]),
        };
        let mut ui = TestUI {};
        let result = fixture.do_install(&config, &mut ui);
        assert!(PathBuf::from(fixture.link).exists());
        assert!(result.is_err())
    }

    #[test]
    fn test_do_install_invalid_file() {
        let fixture = install_fixture(None, "./test_do_install_invalid_files.txt");
        let config = ConfigItem {
            mappings: vec![format!("./foo_bar.toml:{}", fixture.link)],
            before: None,
            after: Some(vec![
                String::from("invalid_cmd test_before > /dev/null"),
                String::from("echo test_before1 > /dev/null"),
            ]),
        };
        let mut ui = TestUI {};
        let result = fixture.do_install(&config, &mut ui);
        assert!(result.is_err());
        assert!(!PathBuf::from(fixture.link).exists());
    }

    #[test]
    fn test_do_install_skip_hooks() {
        let fixture = install_fixture(
            Some(InstallArgs {
                force: false,
                parent: false,
                dry_run: false,
                items: vec![String::from("zsh")],
                skip_hooks: true,
            }),
            "./test_do_install_skip_hooks.txt",
        );
        let config = ConfigItem {
            mappings: vec![format!("./Cargo.toml:{}", fixture.link)],
            before: Some(vec![String::from("touch pre_install.txt")]),
            after: Some(vec![String::from("touch post_install.txt")]),
        };
        let mut ui = TestUI {};
        let res = fixture.do_install(&config, &mut ui);
        assert!(res.is_ok());
        assert!(!PathBuf::from("pre_install.txt").exists());
        assert!(!PathBuf::from("post_install.txt").exists());
    }
}
