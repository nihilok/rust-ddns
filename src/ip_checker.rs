use command_line;
use std::{net::Ipv4Addr, str::FromStr};
use reqwest;

use crate::{error, logging};

const V4_URL: &'static str = "https://api4.ipify.org";

/// Struct that encapsulates all the necessary state and implementations needed for
/// comparing the actual IP of the server it's running on against the current DNS records
#[derive(Debug)]
pub struct IP {
    actual: Option<Ipv4Addr>,
}

impl IP {
    /// Retrieves the current IP address of the host machine.
    ///
    /// This function makes an HTTP GET request to the endpoint defined by `V4_URL`
    /// constant (by default <https://api4.ipify.org>) and retrieves the current IP address.
    ///
    /// The IP address is determined by the outbound connection from the system (client)
    /// to the ipify service, as it perceives it.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// * `Ok` - A string that represent the current IP address of the machine. When the
    /// function succeeds, it returns this variant containing the IP address.
    /// * `Err` - Contains a `reqwest::Error` when there's a failure in getting
    /// the IP address. For example, if there is no network connection or the ipify
    /// service is down.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// async {
    ///     match IP::get_actual_ip().await {
    ///         Ok(ip) => println!("The actual IP is: {}", ip),
    ///         Err(e) => eprintln!("Failed to get the actual IP: {:?}", e),
    ///     }
    /// };
    /// ```
    pub async fn get_actual_ip() -> Result<String, reqwest::Error> {
        Ok(reqwest::get(V4_URL).await?.text().await?)
    }

    /// Checks the current IP for a domain according to DNS records (depends on dig being installed on the current system)
    ///
    /// # Arguments
    ///
    /// * `domain` - A string slice that holds the domain for which we want the IP
    ///
    /// # Errors
    ///
    /// This function will return an error if the dig command fails
    ///
    /// # Example
    ///
    /// ```
    /// let ip = IP::get_previous_ip("https://example.com").await;
    /// println!("{:?}", ip);
    /// ```
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

    /// Compares the current domain's IP with the host's actual IP.
    ///
    /// # Arguments
    ///
    /// * `domain` - A string slice that holds the domain for which we want to compare the host's IP.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// * `Ok` - `true` if the domain IP and the host's actual IP are different, `false` if they are the same.
    /// * `Err` - Contains a `DynamicError` if it fails to compare the IPs.
    ///
    /// # Errors
    ///
    /// This function will return a `Box<dyn std::error::Error>` (or a `DynamicError`) if checking the DNS fails
    /// or if IP address from the string conversion fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let comparison = IP::compare("https://example.com").await;
    /// match comparison {
    ///     Ok(result) => {
    ///         if result {
    ///             println!("The IP has changed.");
    ///         } else {
    ///             println!("The IP has not changed.");
    ///         }
    ///     },
    ///     Err(e) => println!("Failed to compare the IP addresses: {:?}", e),
    /// }
    /// ```
    pub async fn compare(&self, domain: &str) -> Result<bool, crate::error::DynamicError> {
        let logger = logging::Logger::new();
        let current = match IP::get_previous_ip(domain).await {
            Ok(output) => output,
            Err(err) => return Err(Box::new(err)),
        };
        let current_ip;
        if current == "" {
            current_ip = None
        }
        else{ current_ip = Some(Ipv4Addr::from_str(&current)?) }

        if self.actual != current_ip {
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

    /// Constructs a new instance of the `IP` structure with `actual`
    pub fn new() -> IP {
        IP { actual: None }
    }

    /// Sets the `actual` field of the `IP` structure if it is currently `None`.
    ///
    /// It gets the actual IP using the `get_actual_ip` method.
    /// Once the `actual` field is set, it logs the fetched IP address.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if `actual` field was set successfully, or an `Err` wrapping
    /// a `DynamicError` if getting actual IP or conversion from a string to `Ipv4Addr` failed.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Create a new IP instance and set actual.
    /// let mut ip = IP::new();
    /// match ip.set_actual().await {
    ///     Ok(()) => println!("The actual IP has been set."),
    ///     Err(e) => println!("Failed to set the actual IP: {:?}", e),
    /// }
    /// ```
    pub async fn set_actual(&mut self) -> Result<(), error::DynamicError>  {
        if self.actual.is_none() {
            self.actual =
                Some(Ipv4Addr::from_str(IP::get_actual_ip().await?.as_str())?);
            let logger = crate::logging::Logger::new();
            logger.debug(&format!(
                "ipify returned IP address: '{}'",
                self.actual.unwrap().to_string()
            ));
        }
        Ok(())
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
