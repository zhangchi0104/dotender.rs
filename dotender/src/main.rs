mod utils;

use std::time;

use clap::{Parser, Subcommand};
use dotender_lib::{
    commands::install::Install,
    config::parse_config,
    types::{args::InstallArgs, Command},
};

use dotender_lib::utils::into_absolute_path;

#[derive(Parser, Debug)]
#[command(name = "dotender")]
#[command(author = "Alex Zhang")]
struct Cli {
    #[arg(default_value = "~/.dotfiles/config.toml")]
    #[arg(long, short)]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// create links to dotfiles and execute user defined hooks
    Install(InstallArgs),
}

fn main() {
    let args = Cli::parse();
    let cfg_path =
        into_absolute_path(args.config).expect("Unable to parse the provided config path");

    let mut config = parse_config(cfg_path).expect("Error occured when parsing config");
    let begin = time::Instant::now();
    match args.command {
        Commands::Install(args) => {
            let mut cmd = Install::from(args);
            cmd.run(&mut config)
        }
    };
    let end = time::Instant::now();
    println!("Done in {:.2}s", (end - begin).as_secs_f32());
}
