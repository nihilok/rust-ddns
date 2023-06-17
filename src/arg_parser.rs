use clap::{Parser, command};

#[derive(Debug, Parser)]
#[command(author, version, long_about = "Dynamic DNS Client")]
pub struct Args {
    #[arg(short, long)]
    pub config_file: Option<String>,
    #[arg(short, long)]
    pub ip_file: Option<String>,
}