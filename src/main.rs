use clap::Parser;
use futures::future;
use tokio;

mod api_client;
mod arg_parser;
mod error;
mod logging;
mod time_tools;
mod ip_checker;

#[tokio::main]
async fn main() -> Result<(), error::DynamicError> {
    let args = arg_parser::Args::parse();
    let file = args.config_file.unwrap_or(String::from("ddns.conf"));
    let mut config = api_client::APIClient::from_config_file(file);
    let mut futures = Vec::new();
    for c in config.iter_mut() {
        futures.push(c.make_request());
    }
    future::join_all(futures).await;
    Ok(())
}
