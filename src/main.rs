use futures::future;
use api_client::APIClient;

mod api_client;
mod arg_parser;
mod error;
mod ip_checker;
mod logging;
mod time_tools;

const DEFAULT_CONFIG_FILE: &'static str = ".ddns.conf";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::DynamicError> {
    let mut config = APIClient::from_config_file().await;
    let mut futures = Vec::new();
    for protocol in config.iter_mut() {
        futures.push(protocol.execute());
    }
    future::join_all(futures).await;
    Ok(())
}
