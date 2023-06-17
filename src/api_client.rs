use crate::logging::Logger;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Read,
    net::{IpAddr, Ipv6Addr},
    process,
    str::FromStr, sync::Arc,
};

use reqwest::{header, RequestBuilder};
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    fn new(username: String, password: String) -> Credentials {
        return Self { username, password };
    }

    fn authenticate(&self, client: RequestBuilder) -> RequestBuilder {
        client.basic_auth(&self.username, Some(&self.password))
    }
}

#[derive(Debug)]
enum Method {
    POST,
    PUT,
    DELETE,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Method::POST => {
                write!(f, "POST")
            }
            Method::PUT => {
                write!(f, "PUT")
            }
            Method::DELETE => {
                write!(f, "DELETE")
            }
        }
    }
}

impl FromStr for Method {
    type Err = ();

    fn from_str(input: &str) -> Result<Method, Self::Err> {
        match input.to_lowercase().as_str() {
            "post" => Ok(Method::POST),
            "put" => Ok(Method::PUT),
            "delete" => Ok(Method::DELETE),
            _ => Err(()),
        }
    }
}
#[derive(Debug)]
enum Record {
    A,
    AAAA,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Record::A => {
                write!(f, "A")
            }
            Record::AAAA => {
                write!(f, "AAAA")
            }
        }
    }
}

impl FromStr for Record {
    type Err = ();

    fn from_str(input: &str) -> Result<Record, Self::Err> {
        match input.to_lowercase().as_str() {
            "a" => Ok(Record::A),
            "aaaa" => Ok(Record::AAAA),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Protocol {
    GoogleDomains,
    MailInABox,
}

impl Protocol {
    fn build_url(&self, server: &str, domain: &str, record: &str) -> String {
        match self {
            Protocol::GoogleDomains => format!("https://{server}/nic/update?hostname={domain}"),
            Protocol::MailInABox => {
                format!("https://{server}/admin/dns/custom/{domain}/{record}")
            }
        }
    }

    fn from_server(server: &str) -> Self {
        match server {
            "domains.google.com" => Self::GoogleDomains,
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
    checker: Arc<crate::ip_checker::IP>,
    logger: Logger,
}

impl APIClient {
    fn new(
        server: &str,
        domain: &str,
        methods: Vec<&str>,
        records: Vec<&str>,
        credentials: Credentials,
        checker: Arc<crate::ip_checker::IP>,
    ) -> Self {
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

        return Self {
            domain: domain.to_string(),
            server: server.to_string(),
            methods,
            records,
            credentials,
            protocol,
            checker,
            logger,
        };
    }

    pub async fn make_request(&self) -> Result<(), crate::error::DynamicError> {
        let changed = &self.checker.compare(&self.domain).await?;
        if !changed {
            return Ok(());
        }
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
                Record::AAAA => client_builder
                    .local_address(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)))
                    .build()?,
            };

            for method in &self.methods {
                match method {
                    Method::POST => {
                        let client = client.post(&request_url);
                        self.manage_request(client, method, record).await?;
                    }
                    Method::DELETE => {
                        let client = client.delete(&request_url);
                        self.manage_request(client, method, record).await?;
                    }
                    Method::PUT => {
                        let client = client.put(&request_url);
                        self.manage_request(client, method, record).await?;
                    }
                };
            }
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

    fn load_file(file: &str) -> Vec<Yaml> {
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
        checker.set_actual().await;
        let checker = Arc::new(checker);
        let logger = Logger::new();
        let mut config = Vec::new();
        for doc in docs.iter() {
            let username;
            let password;
            let server;
            let domain;
            match doc["username"].as_str() {
                Some(result) => username = result,
                None => {
                    logger.error(&format!("'username' should be in {}", file));
                    process::exit(1);
                }
            };
            match doc["password"].as_str() {
                Some(result) => password = result,
                None => {
                    logger.error(&format!("'password' should be in {}", file));
                    process::exit(1);
                }
            };
            match doc["server"].as_str() {
                Some(result) => server = result,
                None => {
                    logger.error(&format!("'server' should be in {}", file));
                    process::exit(1);
                }
            };
            match doc["domain"].as_str() {
                Some(result) => domain = result,
                None => {
                    logger.error(&format!("'domain' should be in {}", file));
                    process::exit(1);
                }
            };

            let credentials = Credentials::new(username.to_string(), password.to_string());

            let methods: Vec<&str>;

            match doc["methods"].as_vec() {
                Some(methods_vec) => {
                    methods = methods_vec
                        .iter()
                        .map(|m| match m.as_str() {
                            Some(method) => method,
                            None => {
                                logger
                                    .error(&format!("could not parse 'methods' list in {}", file));
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
            let checker_clone = Arc::clone(&checker);
            let api = APIClient::new(server, domain, methods, records, credentials, checker_clone);
            config.push(api)
        }
        config
    }

    pub async fn from_config_file(filename: String) -> Vec<APIClient> {
        let yaml = APIClient::load_file(&filename);
        let yaml = yaml.clone();
        APIClient::parse_yaml(yaml, filename).await
    }
}
