use reqwest::Response;
use tokio;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    fs::File,
    io::Read, net::IpAddr, str::FromStr,
};
use yaml_rust::{YamlLoader, Yaml};
use clap::Parser;


#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file: Option<String>
}


#[derive(Debug)]
struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    fn new(username: String, password: String) -> Credentials {
        return Self { username, password };
    }
}


#[derive(Debug)]
enum Method {
    POST,
    PUT,
    DELETE,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Method::POST => {
                write!(f, "POST")
            },
            Method::PUT => {
                write!(f, "PUT")
            },
            Method::DELETE => {
                write!(f, "DELETE")
            }
        }
    }
}


#[derive(Debug)]
struct API {
    url: String,
    domain: String,
    method: Method,
    credentials: Credentials,
}

impl API {
    fn new(url: &str, domain: &str, method: &str, credentials: Credentials) -> API {
        let m = match method.to_lowercase().as_str() {
            "post" => Method::POST,
            "put" => Method::PUT,
            "delete" => Method::DELETE,
            _ => panic!("method must be PUT, POST, or DELETE"),
        };
        return Self {
            url: url.to_string(),
            domain: domain.to_string(),
            method: m,
            credentials,
        };
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = args.config_file.unwrap_or(String::from("config.yaml"));
    let config = parse_yaml(load_file(&file), &file);
    for c in config.iter() {
        let response = make_request(&c).await?;
        print!("{} {} {}", c.domain, c.method, response.text().await?);
    }
    Ok(())
}

fn load_file(file: &str) -> Vec<Yaml> {
    let mut handle = File::open(file).expect("Unable to open file");
    let mut contents = String::new();

    handle.read_to_string(&mut contents)
        .expect("Unable to read file");

    YamlLoader::load_from_str(&contents).expect("Unable to parse YAML")
}

fn parse_yaml(docs: Vec<Yaml>, file: &str) -> Vec<API> {
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
        let api = API::new(
            doc["url"].as_str().expect(&format!("url should be in {file}")),
        doc["domain"]
            .as_str()
            .expect(&format!("domain should be in {file}")),
        doc["method"]
            .as_str()
            .expect(&format!("method (PUT/POST/DELETE) should be in {file}")),
        credentials,
    );
        config.push(api)
    }
    config
}

async fn make_request(api: &API) -> Result<Response, Box<dyn std::error::Error>> {
    let mut request_url = String::new();
    request_url.push_str(&api.url);
    request_url.push_str(&api.domain);
    request_url.push_str("/A");
    let client = reqwest::Client::builder().local_address(IpAddr::from_str("0.0.0.0")?).build()? ;
    match api.method {
        Method::POST => Ok(client
            .post(&request_url)
            .basic_auth(&api.credentials.username, Some(&api.credentials.password))
            .send()
            .await?),
        Method::DELETE => Ok(client
            .delete(&request_url)
            .basic_auth(&api.credentials.username, Some(&api.credentials.password))
            .send()
            .await?),
        Method::PUT => Ok(client
            .put(&request_url)
            .basic_auth(&api.credentials.username, Some(&api.credentials.password))
            .send()
            .await?),
    }
}
