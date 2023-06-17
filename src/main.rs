use clap::Parser;
use futures::future;
use tokio;

mod api_client;
mod arg_parser;
mod error;
mod ip_checker;
mod logging;
mod time_tools;

const DEFAULT_CONFIG_FILE: &'static str = ".ddns.conf";

#[tokio::main]
async fn main() -> Result<(), error::DynamicError> {
    let file = get_config_file_path();
    let mut config = api_client::APIClient::from_config_file(file);
    let mut futures = Vec::new();
    for c in config.iter_mut() {
        futures.push(c.make_request());
    }
    future::join_all(futures).await;
    Ok(())
}

fn get_config_file_path() -> String {
    let logger = logging::Logger::new();
    let args = arg_parser::Args::parse();
    let mut path = std::env::var("HOME").unwrap_or("".to_string());
    build_config_path(&mut path);
    let file = args.config_file.unwrap_or(path);
    logger.debug(&format!("Using config file '{}'", &file));
    file
}

#[cfg(not(target_os = "windows"))]
fn build_config_path(path: &mut String) {
    if path.len() > 0 {
        path.push_str("/")
    }
    path.push_str(DEFAULT_CONFIG_FILE);
}

#[cfg(target_os = "windows")]
fn build_config_path(path: &mut String) {
    if path.len() > 0 {
        path.push_str("\\")
    }
    path.push_str(DEFAULT_CONFIG_FILE);
}
