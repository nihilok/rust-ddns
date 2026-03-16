use clap::{command, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, long_about = "Dynamic DNS Client")]
pub struct Args {
    #[arg(short, long)]
    pub config_file: Option<String>,
    #[arg(short, long)]
    pub ip: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Install {
        #[arg(long, default_value = "5min")]
        interval: String,
        #[arg(long)]
        log_file: Option<String>,
        #[arg(long)]
        config_file: Option<String>,
    },
    Uninstall {
        #[arg(long, default_value_t = false)]
        purge: bool,
    },
}
