use crate::logging::Logger;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Read,
    net::{IpAddr, Ipv6Addr},
    process,
    str::FromStr,
    rc::Rc,
};

use futures::future;
use reqwest::{header, RequestBuilder};
use yaml_rust::{Yaml, YamlLoader};

use crate::ip_checker;

#[derive(Debug)]
struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    fn new(username: String, password: String) -> Credentials {
        Self { username, password }
    }

    fn authenticate(&self, client: RequestBuilder) -> RequestBuilder {
        client.basic_auth(&self.username, Some(&self.password))
    }
}

#[derive(Debug)]
enum Method {
    Post,
    Put,
    Delete,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
        }
    }
}

impl FromStr for Method {
    type Err = ();

    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input.to_lowercase().as_str() {
            "post" => Ok(Method::Post),
            "put" => Ok(Method::Put),
            "delete" => Ok(Method::Delete),
            _ => Err(()),
        }
    }
}
#[derive(Debug)]
enum Record {
    A,
    Aaaa,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Record::A => write!(f, "A"),
            Record::Aaaa => write!(f, "AAAA"),
        }
    }
}

impl FromStr for Record {
    type Err = ();

    fn from_str(input: &str) -> Result<Record, Self::Err> {
        match input.to_lowercase().as_str() {
            "a" => Ok(Record::A),
            "aaaa" => Ok(Record::Aaaa),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Protocol {
    Cloudflare,
    Namecheap,
    MailInABox,
}

impl Protocol {
    fn build_url(&self, server: &str, domain: &str, record: &str) -> String {
        match self {
            Protocol::Cloudflare => String::new(),
            Protocol::Namecheap => String::new(),
            Protocol::MailInABox => {
                format!("https://{server}/admin/dns/custom/{domain}/{record}")
            }
        }
    }

    fn from_server(server: &str) -> Self {
        match server {
            "domains.google.com" => {
                eprintln!("ERROR: Google Domains DDNS (domains.google.com) is no longer supported. Google sold Domains to Squarespace, which dropped DDNS support. Please migrate to Cloudflare: update your config to use 'server: cloudflare' with an 'api_token'. See README for migration steps.");
                std::process::exit(1);
            }
            "cloudflare" => Self::Cloudflare,
            "namecheap" => Self::Namecheap,
            _ => Self::MailInABox,
        }
    }
}

#[derive(Debug)]
pub struct APIClient {
    domain: String,
    methods: Vec<Method>,
    records: Vec<Record>,
    credentials: Credentials,
    server: String,
    protocol: Protocol,
    api_token: Option<String>,
    checker: Rc<ip_checker::IP>,
    logger: Logger,
}

impl APIClient {
    fn new(
        server: &str,
        domain: &str,
        methods: Vec<&str>,
        records: Vec<&str>,
        credentials: Credentials,
        api_token: Option<String>,
        checker: Rc<ip_checker::IP>,
        ) -> APIClient {
        let logger = Logger::new();

        let methods: Vec<Method> = methods
            .iter()
            .map(|x| match Method::from_str(x) {
                Ok(m) => m,
                Err(_) => {
                    logger.error(&format!(
                        "Could not parse methods in config file; must be PUT, POST or DELETE (got '{}')",
                        x
                    ));
                    process::exit(1);
                }
            })
            .collect();

        let records: Vec<Record> = records
            .iter()
            .map(|x| match Record::from_str(x) {
                Ok(r) => r,
                Err(_) => {
                    logger.error(&format!(
                        "Could not parse records in config file; must be A or AAAA (got '{}')",
                        x
                    ));
                    process::exit(1);
                }
            })
            .collect();

        let protocol = Protocol::from_server(server);

        Self {
            domain: domain.to_string(),
            server: server.to_string(),
            methods,
            records,
            credentials,
            protocol,
            api_token,
            checker,
            logger,
        }
    }

    pub async fn execute(&self) -> Result<(), crate::error::DynamicError> {
        let changed = self.checker.compare(&self.domain).await?;
        if !changed {
            return Ok(());
        }

        if self.protocol == Protocol::Cloudflare {
            return self.execute_cloudflare().await;
        }

        if self.protocol == Protocol::Namecheap {
            return self.execute_namecheap().await;
        }
        let mut calls = Vec::new();
        for record in &mut self.records.iter() {
            let request_url =
                self.protocol
                    .build_url(&self.server, &self.domain, &record.to_string());
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::USER_AGENT,
                header::HeaderValue::from_static("Rust Reqwest"),
            );
            headers.insert(
                header::CONTENT_LENGTH,
                header::HeaderValue::from_static("0"),
            );
            let client_builder = reqwest::Client::builder().default_headers(headers);
            let client = match record {
                Record::A => client_builder
                    .local_address(IpAddr::from_str("0.0.0.0")?)
                    .build()?,
                Record::Aaaa => client_builder
                    .local_address(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)))
                    .build()?,
            };
            calls.push(self.call_all_methods(client, request_url, record))
        }
        future::join_all(calls).await;
        Ok(())
    }

    async fn execute_cloudflare(&self) -> Result<(), crate::error::DynamicError> {
        let token = match &self.api_token {
            Some(t) => t.clone(),
            None => {
                return Err("Cloudflare api_token is required".into());
            }
        };

        let ip = match self.checker.actual_ip() {
            Some(ip) => ip,
            None => {
                return Err("Could not determine actual IP".into());
            }
        };

        let apex_domain = apex_domain_from(&self.domain);

        let client = reqwest::Client::new();

        for record in &self.records {
            let record_type = record.to_string();

            // Resolve zone ID
            let zone_url = format!(
                "https://api.cloudflare.com/client/v4/zones?name={}",
                apex_domain
            );
            let zone_resp = client
                .get(&zone_url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let zone_id = zone_resp["result"][0]["id"]
                .as_str()
                .ok_or_else(|| format!("Could not find Cloudflare zone for domain '{}'", apex_domain))?
                .to_string();

            // Resolve record ID
            let records_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type={}",
                zone_id, self.domain, record_type
            );
            let records_resp = client
                .get(&records_url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let record_id = records_resp["result"][0]["id"]
                .as_str()
                .ok_or_else(|| format!("Could not find Cloudflare DNS record for '{}' type {}", self.domain, record_type))?
                .to_string();

            // Update record
            let update_url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            );
            let body = serde_json::json!({
                "type": record_type,
                "name": self.domain,
                "content": ip.to_string(),
                "ttl": 1
            });

            let update_resp = client
                .put(&update_url)
                .header("Authorization", format!("Bearer {}", token))
                .json(&body)
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            if update_resp["success"].as_bool().unwrap_or(false) {
                self.logger.info(&format!(
                    "{} {} Cloudflare updated to {}",
                    self.domain, record_type, ip
                ));
            } else {
                let errors = update_resp["errors"].to_string();
                self.logger.error(&format!(
                    "{} {} Cloudflare update failed: {}",
                    self.domain, record_type, errors
                ));
                return Err(format!("Cloudflare update failed: {}", errors).into());
            }
        }

        Ok(())
    }

    async fn execute_namecheap(&self) -> Result<(), crate::error::DynamicError> {
        for record in &self.records {
            if let Record::Aaaa = record {
                let msg = "Namecheap DDNS does not support AAAA records";
                self.logger.error(msg);
                return Err(msg.into());
            }
        }

        let ip = match self.checker.actual_ip() {
            Some(ip) => ip,
            None => return Err("Could not determine actual IP".into()),
        };

        let parts: Vec<&str> = self.domain.splitn(2, '.').collect();
        let (host, domain) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("@".to_string(), self.domain.clone())
        };

        let password = &self.credentials.password;
        let client = reqwest::Client::new();
        let resp = client
            .get("https://dynamicdns.park-your-domain.com/update")
            .query(&[
                ("host", host.as_str()),
                ("domain", domain.as_str()),
                ("password", password.as_str()),
                ("ip", &ip.to_string()),
            ])
            .send()
            .await?
            .text()
            .await?;

        if resp.contains("<ErrCount>0</ErrCount>") {
            self.logger.info(&format!(
                "{} A Namecheap updated to {}",
                self.domain, ip
            ));
        } else {
            let err_text = extract_xml_tag(&resp, "Err1")
                .unwrap_or_else(|| resp.clone());
            self.logger.error(&format!(
                "{} A Namecheap update failed: {}",
                self.domain, err_text
            ));
            return Err(format!("Namecheap update failed: {}", err_text).into());
        }

        Ok(())
    }

    async fn call_all_methods(&self, client: reqwest::Client, url: String, record: &Record) -> Result<(), reqwest::Error> {
        for method in &self.methods {
            match method {
                Method::Post => {
                    let client = client.post(&url);
                    self.manage_request(client, method, record).await?;
                }
                Method::Delete => {
                    let client = client.delete(&url);
                    self.manage_request(client, method, record).await?;
                }
                Method::Put => {
                    let client = client.put(&url);
                    self.manage_request(client, method, record).await?;
                }
            };
        }
        Ok(())
    }

    async fn manage_request(
        &self,
        client: RequestBuilder,
        method: &Method,
        record: &Record,
    ) -> Result<(), reqwest::Error> {
        let client = self.credentials.authenticate(client);
        let resp = client.send().await?;
        let text = resp.text().await?;
        self.logger
            .info(&format!("{} {} {} {}", &self.domain, record, method, text));
        Ok(())
    }

    pub async fn from_config_file(filename: String) -> Vec<APIClient> {
        let yaml = load_yaml_from_file(&filename);
        let yaml = yaml.clone();
        parse_yaml(yaml, filename).await
    }
}

fn apex_domain_from(domain: &str) -> String {
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() >= 3 {
        parts[parts.len() - 2..].join(".")
    } else {
        domain.to_string()
    }
}

fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].to_string())
}

fn resolve_secret(value: &str) -> Result<String, crate::error::DynamicError> {
    if let Some(var_name) = value.strip_prefix("env:") {
        match std::env::var(var_name) {
            Ok(v) if !v.is_empty() => Ok(v),
            _ => Err(format!("password env var '{}' is not set", var_name).into()),
        }
    } else {
        Ok(value.to_string())
    }
}

fn load_yaml_from_file(file: &str) -> Vec<Yaml> {
    let logger = crate::logging::Logger::new();
    let mut handle = match File::open(file) {
        Ok(f) => f,
        Err(_) => {
            logger.error(&format!("Could not load config file {}", file));
            std::process::exit(1)
        }
    };
    let mut contents = String::new();

    match handle.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => {
            logger.debug(&format!("{:#?}", err));
            std::process::exit(1)
        }
    }

    YamlLoader::load_from_str(&contents).expect("Unable to parse YAML")
}

async fn parse_yaml(docs: Vec<Yaml>, file: String) -> Vec<APIClient> {
    let mut checker = crate::ip_checker::IP::new();
    match checker.set_actual().await {
        Ok(_) => {}
        Err(err) => {
            let logger = Logger::new();
            logger.error(&format!("{}", err));
            process::exit(1)
        }
    }
    let checker = Rc::new(checker);
    let logger = Logger::new();
    let mut config = Vec::new();
    for doc in docs.iter() {
        let server = match doc["server"].as_str() {
            Some(result) => result,
            None => {
                logger.error(&format!("'server' should be in {}", file));
                process::exit(1);
            }
        };
        let domain = match doc["domain"].as_str() {
            Some(result) => result,
            None => {
                logger.error(&format!("'domain' should be in {}", file));
                process::exit(1);
            }
        };

        let (credentials, api_token) = if server == "cloudflare" {
            let raw_token = match doc["api_token"].as_str() {
                Some(t) => t,
                None => {
                    logger.error(&format!("'api_token' is required for Cloudflare in {}", file));
                    process::exit(1);
                }
            };
            let token = match resolve_secret(raw_token) {
                Ok(v) => v,
                Err(e) => { logger.error(&format!("{}", e)); process::exit(1); }
            };
            (Credentials::new(String::new(), String::new()), Some(token))
        } else if server == "namecheap" {
            let raw_password = match doc["password"].as_str() {
                Some(result) => result,
                None => {
                    logger.error(&format!("'password' is required for Namecheap in {}", file));
                    process::exit(1);
                }
            };
            let password = match resolve_secret(raw_password) {
                Ok(v) => v,
                Err(e) => { logger.error(&format!("{}", e)); process::exit(1); }
            };
            (Credentials::new(String::new(), password), None)
        } else {
            let username = match doc["username"].as_str() {
                Some(result) => result,
                None => {
                    logger.error(&format!("'username' should be in {}", file));
                    process::exit(1);
                }
            };
            let password = match doc["password"].as_str() {
                Some(result) => result,
                None => {
                    logger.error(&format!("'password' should be in {}", file));
                    process::exit(1);
                }
            };
            let username = match resolve_secret(username) {
                Ok(v) => v,
                Err(e) => { logger.error(&format!("{}", e)); process::exit(1); }
            };
            let password = match resolve_secret(password) {
                Ok(v) => v,
                Err(e) => { logger.error(&format!("{}", e)); process::exit(1); }
            };
            (Credentials::new(username, password), None)
        };

        let methods: Vec<&str> = if server == "cloudflare" || server == "namecheap" {
            // Cloudflare and Namecheap don't use methods, use a placeholder
            vec!["put"]
        } else {
            match doc["methods"].as_vec() {
                Some(methods_vec) => {
                    methods_vec
                        .iter()
                        .map(|m| match m.as_str() {
                            Some(method) => method,
                            None => {
                                logger.error(&format!("could not parse 'methods' list in {}", file));
                                process::exit(1);
                            }
                        })
                        .collect()
                }
                None => {
                    logger.error(&format!("'methods' (list) should be in {}", file));
                    process::exit(1);
                }
            }
        };

        let records = doc["records"].as_vec();
        let records = match records {
            Some(v) => v
                    .iter()
                    .map(|r| match r.as_str() {
                        Some(record) => record,
                        None => {
                            logger.error(&format!("could not parse 'records' list in {}", file));
                            process::exit(1);
                        }
                    })
                    .collect(),
            None => vec!["a"],
        };
        let checker_clone = Rc::clone(&checker);
        let api = APIClient::new(server, domain, methods, records, credentials, api_token, checker_clone);
        config.push(api)
    }
    config
}

pub fn get_config_file_path(user_file_path: Option<String>) -> String {
    let logger = Logger::new();
    let file_path = user_file_path.unwrap_or_else(|| {
        let mut path = std::env::var("HOME").unwrap_or("".to_string());
        build_config_path(&mut path);
        path
    });
    logger.debug(&format!("Using config file '{}'", &file_path));
    file_path
}

#[cfg(not(target_os = "windows"))]
fn build_config_path(path: &mut String) {
    use crate::DEFAULT_CONFIG_FILE;

    if !path.is_empty() {
        path.push('/');
    }
    path.push_str(DEFAULT_CONFIG_FILE);
}

#[cfg(target_os = "windows")]
fn build_config_path(path: &mut String) {
    use crate::DEFAULT_CONFIG_FILE;

    if !path.is_empty() {
        path.push('\\');
    }
    path.push_str(DEFAULT_CONFIG_FILE);
}
