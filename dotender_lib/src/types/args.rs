use clap::Args;

#[derive(Args, Debug, Default)]
pub struct InstallArgs {
    /// Override the link if already exisits
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
    /// Creates the symlolic link's parent dir if not exist
    #[arg(short, long, default_value_t = false)]
    pub parent: bool,
    /// show install plan without hooks and links
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,
    /// name of the symbolic links
    pub items: Vec<String>,
    /// skip pre and post install hooks
    #[arg(long, default_value_t = false)]
    pub skip_hooks: bool,
}

pub type LinkArgs = InstallArgs;
