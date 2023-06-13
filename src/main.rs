use clap::Parser;
use std::{fs::File, io::Read, net::IpAddr, str::FromStr};
use tokio;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file: Option<String>,
}

mod api {
    use std::{
        fmt::{Display, Formatter},
        str::FromStr,
    };

    #[derive(Debug)]
    pub struct Credentials {
        pub username: String,
        pub password: String,
    }

    impl Credentials {
        pub fn new(username: String, password: String) -> Credentials {
            return Self { username, password };
        }
    }

    #[derive(Debug)]
    pub enum Method {
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
    pub struct API {
        pub url: String,
        pub domain: String,
        pub method: Vec<Method>,
        pub credentials: Credentials,
    }

    impl API {
        pub fn new(url: &str, domain: &str, methods: Vec<&str>, credentials: Credentials) -> API {
            let methods: Vec<Method> = methods
                .iter()
                .map(|x| {
                    Method::from_str(x)
                        .expect("Could not parse method, must be PUT, POST or DELETE")
                })
                .collect();
            return Self {
                url: url.to_string(),
                domain: domain.to_string(),
                method: methods,
                credentials,
            };
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = args.config_file.unwrap_or(String::from("config.yaml"));
    let config = parse_yaml(load_file(&file), &file);
    for c in config.iter() {
        make_request(&c).await?;
    }
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

fn parse_yaml(docs: Vec<Yaml>, file: &str) -> Vec<api::API> {
    let mut config = Vec::new();
    for doc in docs.iter() {
        let credentials = api::Credentials::new(
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
            .map(|m| m.as_str().unwrap())
            .collect();
        let api = api::API::new(
            doc["url"]
                .as_str()
                .expect(&format!("url should be in {file}")),
            doc["domain"]
                .as_str()
                .expect(&format!("domain should be in {file}")),
            methods,
            credentials,
        );
        config.push(api)
    }
    config
}

async fn make_request(api: &api::API) -> Result<(), Box<dyn std::error::Error>> {
    let mut request_url = String::new();
    request_url.push_str(&api.url);
    request_url.push_str(&api.domain);
    request_url.push_str("/A");
    let client = reqwest::Client::builder()
        .local_address(IpAddr::from_str("0.0.0.0")?)
        .build()?;
    for m in &api.method {
        match m {
            api::Method::POST => {
                let resp = client
                    .post(&request_url)
                    .basic_auth(&api.credentials.username, Some(&api.credentials.password))
                    .send()
                    .await?;
                print!("{} {} {}", &api.domain, m, resp.text().await?)
            }
            api::Method::DELETE => {
                let resp = client
                    .delete(&request_url)
                    .basic_auth(&api.credentials.username, Some(&api.credentials.password))
                    .send()
                    .await?;
                print!("{} {} {}", &api.domain, m, resp.text().await?)
            }
            api::Method::PUT => {
                let resp = client
                    .put(&request_url)
                    .basic_auth(&api.credentials.username, Some(&api.credentials.password))
                    .send()
                    .await?;
                print!("{} {} {}", &api.domain, m, resp.text().await?)
            }
        };
    }
    Ok(())
}
