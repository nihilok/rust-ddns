use crate::logging;
use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::Read,
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
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
}

impl APIClient {
    fn new(
        server: &str,
        domain: &str,
        methods: Vec<&str>,
        records: Vec<&str>,
        credentials: Credentials,
    ) -> Self {
        let methods: Vec<Method> = methods
            .iter()
            .map(|x| {
                Method::from_str(x).expect("Could not parse method, must be PUT, POST or DELETE")
            })
            .collect();

        let records: Vec<Record> = records
            .iter()
            .map(|x| Record::from_str(x).expect("Could not parse record, must be either A or AAAA"))
            .collect();

        let protocol = Protocol::from_server(server);

        return Self {
            domain: domain.to_string(),
            server: server.to_string(),
            methods,
            records,
            credentials,
            protocol,
        };
    }

    pub async fn make_request(&self) -> Result<(), Box<dyn std::error::Error>> {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let logger = logging::Logger::new();
        let client = self.credentials.authenticate(client);
        let resp = client.send().await?;
        let text = resp.text().await?;
        logger.info(&format!("{} {} {} {}", &self.domain, record, method, text));
        Ok(())
    }

    fn load_file(file: &str) -> Vec<Yaml> {
        let mut handle = File::open(file).expect("Unable to open file");
        let mut contents = String::new();

        handle
            .read_to_string(&mut contents)
            .expect("Unable to read file");

        YamlLoader::load_from_str(&contents).expect("Unable to parse YAML")
    }

    fn parse_yaml(docs: Vec<Yaml>, file: String) -> Vec<APIClient> {
        let mut config = Vec::new();
        for doc in docs.iter() {
            let credentials = Credentials::new(
                doc["username"]
                    .as_str()
                    .expect(&format!("username should be in {file}"))
                    .to_string(),
                doc["password"]
                    .as_str()
                    .expect(&format!("password should be in {file}"))
                    .to_string(),
            );
            let methods: Vec<&str> = doc["methods"]
                .as_vec()
                .expect(&format!(
                    "method list (PUT/POST/DELETE) should be in {file}"
                ))
                .iter()
                .map(|m| m.as_str().expect("should be able to parse methods list"))
                .collect();
            let records = doc["records"].as_vec();
            let records = match records {
                Some(v) => v
                    .iter()
                    .map(|m| {
                        m.as_str()
                            .expect(&format!("should be able to parse records list in {file}"))
                    })
                    .collect(),
                None => vec!["a"],
            };

            let api = APIClient::new(
                doc["server"]
                    .as_str()
                    .expect(&format!("server should be in {file}")),
                doc["domain"]
                    .as_str()
                    .expect(&format!("domain should be in {file}")),
                methods,
                records,
                credentials,
            );
            config.push(api)
        }
        config
    }

    pub fn from_config_file(filename: String) -> Vec<APIClient> {
        let yaml = APIClient::load_file(&filename);
        APIClient::parse_yaml(yaml, filename)
    }
}
