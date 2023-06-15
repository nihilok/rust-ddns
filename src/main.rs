use clap::Parser;
use futures::future::join_all;
use tokio;

mod api_client;
mod ip_checker;
mod time_tools;
mod logging;

const LOG_LEVEL: logging::LogLevel = logging::LogLevel::INFO;

type DynamicError = Box<dyn std::error::Error>;

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file: Option<String>,
    #[arg(short, long)]
    ip_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), DynamicError> {
    let args = Args::parse();
    let ip_file = args.ip_file.unwrap_or(String::from(".ip"));
    let mut ip = ip_checker::IP::load(ip_file);
    ip.compare().await?;
    if !ip.changed {
        return Ok(());
    }
    let file = args.config_file.unwrap_or(String::from("config.yaml"));
    let config = api_client::APIClient::from_config_file(file);
    let mut futures = Vec::new();
    for c in config.iter() {
        futures.push(c.make_request());
    }
    join_all(futures).await;
    Ok(())
}
