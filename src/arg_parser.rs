use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(author, version, long_about = "Dynamic DNS Client")]
pub struct Args {
    #[arg(short, long)]
    pub config_file: Option<String>,
}
