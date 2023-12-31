use std::future::Future;
use futures::future;
use api_client::APIClient;
use arg_parser::Args;
use clap::Parser;

mod api_client;
mod arg_parser;
mod error;
mod ip_checker;
mod logging;
mod time_tools;

const DEFAULT_CONFIG_FILE: &'static str = ".ddns.conf";


async fn log_and_ignore_errors<F>(fut: F)
    where
        F: Future<Output = Result<(), error::DynamicError>>,
{
    let logger = crate::logging::Logger::new();
    match fut.await {
        Ok(_) => {}
        Err(err) => {
            logger.error(&format!("{}", err))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), error::DynamicError> {
    let args = Args::parse();

    if args.ip {
        println!("{}", ip_checker::IP::get_actual_ip().await?);
        return Ok(())
    }

    let file = api_client::get_config_file_path(args.config_file);
    let mut config = APIClient::from_config_file(file).await;
    let mut futures = Vec::new();
    for protocol in config.iter_mut() {
        futures.push(log_and_ignore_errors(protocol.execute()));
    }
    future::join_all(futures).await;
    Ok(())
}
