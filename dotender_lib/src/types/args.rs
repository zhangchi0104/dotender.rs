use clap::Args;

#[derive(Args, Debug, Default)]
pub struct InstallArgs {
    /// Override the link if already exisits
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
    /// Creates the symlolic link's parent dir if not exist
    #[arg(short, long, default_value_t = false)]
    pub parent: bool,
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,
    /// name of the symbolic links
    pub items: Vec<String>,
}
