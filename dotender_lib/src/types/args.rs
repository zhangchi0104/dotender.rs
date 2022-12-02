use clap::Args;

#[derive(Args)]
pub struct InstallArgs {
    pub force: bool,
    pub parent: bool,
}
