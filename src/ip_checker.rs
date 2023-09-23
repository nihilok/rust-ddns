use command_line;
use std::{net::Ipv4Addr, str::FromStr};
use reqwest;

use crate::logging;

const V4_URL: &'static str = "https://api4.ipify.org";

#[derive(Debug)]
pub struct IP {
    actual: Option<Ipv4Addr>,
}

impl IP {
    pub async fn get_actual_ip() -> Result<String, reqwest::Error> {
        Ok(reqwest::get(V4_URL).await?.text().await?)
    }

    async fn get_previous_ip(domain: &str) -> Result<String, command_line::Errors> {
        let logger = logging::Logger::new();
        match command_line::sh(&format!("dig +short {}", domain)) {
            Ok(output) => {
                let mut result = output;
                trim_newline(&mut result);
                logger.debug(&format!(
                    "dig returned IP address: '{}' for domain: '{}'",
                    result, domain
                ));
                Ok(result)
            }
            Err(err) => {
                logger.error(&format!("dig command failed with output: {}", err));
                Err(err) },
        }
    }

    pub async fn compare(&self, domain: &str) -> Result<bool, crate::error::DynamicError> {
        let logger = logging::Logger::new();
        let current = match IP::get_previous_ip(domain).await {
            Ok(output) => output,
            Err(err) => return Err(Box::new(err)),
        };
        let current_ip = Ipv4Addr::from_str(&current)?;

        if self.actual != Some(current_ip) {
            logger.info(&format!(
                "IP address changed: New IP: {}",
                self.actual.unwrap().to_string()
            ));
            return Ok(true);
        } else {
            logger.debug("IP address did not change");
        }
        Ok(false)
    }

    pub fn new() -> IP {
        IP { actual: None }
    }

    pub async fn set_actual(&mut self) {
        if self.actual.is_none() {
            self.actual =
                Some(Ipv4Addr::from_str(IP::get_actual_ip().await.unwrap().as_str()).unwrap());
            let logger = crate::logging::Logger::new();
            logger.debug(&format!(
                "ipify returned IP address: '{}'",
                self.actual.unwrap().to_string()
            ));
        }
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
