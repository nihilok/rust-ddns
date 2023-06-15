use std::{
    fs::File,
    io::{Read, Write},
    net::Ipv4Addr,
    str::FromStr,
};

use crate::logging;

const V4_URL: &'static str = "https://api.ipify.org";

pub struct IP {
    current: Ipv4Addr,
    pub changed: bool,
    filename: String,
}

impl IP {
    async fn get_actual_ip() -> Result<String, reqwest::Error> {
        Ok(reqwest::get(V4_URL).await?.text().await?)
    }

    pub async fn compare(&mut self) -> Result<(), crate::DynamicError> {
        let logger = logging::Logger::new();
        logger.debug(&format!("Saved IP address '{}'", &self.current));
        logger.debug(&format!("Making request to {}", V4_URL));
        let actual = IP::get_actual_ip().await?;
        logger.debug(&format!("Request returned IP address '{}'", actual));
        let actual_ip = Ipv4Addr::from_str(&actual)?;
        if actual_ip != self.current {
            logger.info(&format!("IP address changed: New IP: {}", actual_ip));
            self.current = actual_ip;
            self.changed = true;
        } else {
            logger.debug("IP address did not change");
        }
        Ok(())
    }

    fn write_current_ip_to_file(&self) -> Result<(), std::io::Error> {
        let mut file = File::create(&self.filename)?;
        file.write_all(format!("{}", self.current).as_bytes())?;
        Ok(())
    }

    pub fn new(filename: &str) -> IP {
        let ip_result = IP::from_file(filename);
        match ip_result {
            Ok(ip) => ip,
            Err(_) => IP {
                current: Ipv4Addr::from_str("0.0.0.0").expect("can create IP from hardcoded str"),
                changed: false,
                filename: filename.to_string(),
            },
        }
    }

    pub fn load(filename: String) -> IP {
        let f = filename.as_str();
        IP::new(f)
    }

    fn from_file(file: &str) -> Result<IP, crate::DynamicError> {
        let handle_result = File::open(file);
        let mut handle = match handle_result {
            Ok(f) => f,
            Err(_) => {
                let mut f = File::create(file)?;
                f.write_all(b"0.0.0.0")?;
                File::open(file)?
            }
        };
        let mut contents = String::new();
        handle.read_to_string(&mut contents)?;
        Ok(IP {
            current: Ipv4Addr::from_str(&contents)?,
            changed: false,
            filename: file.to_string(),
        })
    }
}

impl Drop for IP {
    fn drop(&mut self) {
        let logger = logging::Logger::new();
        match self.write_current_ip_to_file() {
            Ok(_) => (),
            Err(err) => logger.error(&format!("Error writing IP to file\n{:#?}", err)),
        };
    }
}
