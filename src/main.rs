use futures::future;
use tokio;
use api_client::APIClient;

mod api_client;
mod arg_parser;
mod error;
mod ip_checker;
mod logging;
mod time_tools;

const DEFAULT_CONFIG_FILE: &'static str = ".ddns.conf";

#[tokio::main]
async fn main() -> Result<(), error::DynamicError> {
    let file = api_client::get_config_file_path();
    let mut config = APIClient::from_config_file(file).await;
    let mut futures = Vec::new();
    for api in config.iter_mut() {
        futures.push(api.execute_protocol());
    }
    future::join_all(futures).await;
    Ok(())
}
