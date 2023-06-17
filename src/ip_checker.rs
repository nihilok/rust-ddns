use command_line;
use std::{net::Ipv4Addr, str::FromStr};

use crate::logging;

const V4_URL: &'static str = "https://api4.ipify.org";

#[derive(Debug)]
pub struct IP {
    pub domain: String,
}

impl IP {
    async fn get_actual_ip() -> Result<String, reqwest::Error> {
        Ok(reqwest::get(V4_URL).await?.text().await?)
    }

    async fn get_previous_ip(domain: &str) -> Result<String, command_line::Errors> {
        match command_line::execute_command(&format!("dig +short {}", domain)) {
            Ok(output) => {
                let mut result = output.output().to_string();
                trim_newline(&mut result);
                Ok(result)
            }
            Err(_) => todo!(),
        }
    }

    pub async fn compare(&self) -> Result<bool, crate::error::DynamicError> {
        let logger = logging::Logger::new();
        let actual = IP::get_actual_ip().await?;
        let current = match IP::get_previous_ip(&self.domain).await {
            Ok(output) => output,
            Err(err) => return Err(Box::new(err)),
        };
        logger.debug(&format!("dig returned IP address: '{}'", current));
        logger.debug(&format!("ipify returned IP address: '{}'", actual));

        let actual_ip = Ipv4Addr::from_str(&actual)?;
        let current_ip = Ipv4Addr::from_str(&current)?;
        if actual_ip != current_ip {
            logger.info(&format!("IP address changed: New IP: {}", actual_ip));
            return Ok(true);
        } else {
            logger.debug("IP address did not change");
        }
        Ok(false)
    }

    pub fn new(domain: &str) -> IP {
        IP {
            domain: domain.to_string(),
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
