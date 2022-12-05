use std::fmt::Display;

pub struct InstallUIState<'state>(Option<InstallStep<'state>>);

pub enum InstallStep<'step> {
    PreInstallHook(&'step str),
    PostInstallHook(&'step str),
    Link(&'step str, &'step str),
}

impl Display for InstallStep<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallStep::PreInstallHook(cmd) => write!(f, "Running Pre-Install hook: {cmd}"),
            InstallStep::PostInstallHook(cmd) => write!(f, "Running Post-Install hook: {cmd}"),
            InstallStep::Link(origin, link) => write!(f, "linking: {origin} -> {link}"),
        }
    }
}

pub struct InstallUI<'ui> {
    pub(crate) pbar: indicatif::ProgressBar,
    pub(crate) state: InstallUIState<'ui>,
}

impl<'ui> InstallUI<'ui> {
    pub fn new() -> Self {
        let pbar = indicatif::ProgressBar::new_spinner();
        Self {
            pbar,
            state: InstallUIState(None),
        }
    }
    pub fn done_with_msg(&self, msg: &'static str) {
        self.pbar.finish_with_message(msg);
    }
}

impl<'state> super::StatefulUI<InstallStep<'state>> for InstallUI<'state> {
    fn update(&mut self, new_state: InstallStep<'state>) {
        self.state = InstallUIState::from(new_state);
        let _ = self
            .state
            .0
            .as_ref()
            .map(|step| Some(self.pbar.set_message(format!("{step}"))));
        self.pbar.inc(1);
    }
}

impl<'state> From<InstallStep<'state>> for InstallUIState<'state> {
    fn from(step: InstallStep<'state>) -> Self {
        Self(Some(step))
    }
}

impl From<indicatif::ProgressBar> for InstallUI<'_> {
    fn from(pbar: indicatif::ProgressBar) -> Self {
        Self {
            pbar,
            state: InstallUIState(None),
        }
    }
}

impl Default for InstallUI<'_> {
    fn default() -> Self {
        Self::new()
    }
}
