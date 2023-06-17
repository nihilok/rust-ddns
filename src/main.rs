use clap::Parser;
use futures::future;
use tokio;

mod api_client;
mod arg_parser;
mod error;
mod ip_checker;
mod logging;
mod time_tools;

const LOG_LEVEL: logging::LogLevel = logging::LogLevel::INFO;

#[tokio::main]
async fn main() -> Result<(), error::DynamicError> {
    let args = arg_parser::Args::parse();
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
    future::join_all(futures).await;
    Ok(())
}
