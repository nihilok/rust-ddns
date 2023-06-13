use clap::Parser;
use tokio;

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file: Option<String>,
    #[arg(short, long)]
    ip_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let ip_file = args.ip_file.unwrap_or(String::from(".ip"));
    let mut ip = ip_checker::IP::load(ip_file);
    ip.compare().await?;
    if !ip.changed {
        return Ok(());
    }
    let file = args.config_file.unwrap_or(String::from("config.yaml"));
    let config = api::API::from_config_file(&file);
    for c in config.iter() {
        c.make_request().await?;
    }
    Ok(())
}

mod api {
    use std::{
        fmt::{Display, Formatter},
        fs::File,
        io::Read,
        net::{IpAddr, Ipv6Addr},
        str::FromStr,
    };

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
    pub enum Record {
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

    #[derive(Debug)]
    pub struct API {
        url: String,
        domain: String,
        methods: Vec<Method>,
        records: Vec<Record>,
        credentials: Credentials,
    }

    impl API {
        fn new(
            url: &str,
            domain: &str,
            methods: Vec<&str>,
            records: Vec<&str>,
            credentials: Credentials,
        ) -> API {
            let methods: Vec<Method> = methods
                .iter()
                .map(|x| {
                    Method::from_str(x)
                        .expect("Could not parse method, must be PUT, POST or DELETE")
                })
                .collect();

            let records: Vec<Record> = records
                .iter()
                .map(|x| {
                    Record::from_str(x).expect("Could not parse record, must be either A or AAAA")
                })
                .collect();

            return Self {
                url: url.to_string(),
                domain: domain.to_string(),
                methods,
                records,
                credentials,
            };
        }

        fn print_response(request_url: &str, method: &Method, response_text: &str) {
            let newline = if response_text.ends_with("\n") {
                ""
            } else {
                "\n"
            };
            print!("{} {} {}{}", request_url, method, response_text, newline);
        }

        pub async fn make_request(&self) -> Result<(), Box<dyn std::error::Error>> {
            let mut request_url_base = String::new();
            request_url_base.push_str(&self.url);
            request_url_base.push_str(&self.domain);

            for record in &mut self.records.iter() {
                let mut request_url = request_url_base.clone();
                let client_builder = reqwest::Client::builder();
                let client = match record {
                    Record::A => {
                        request_url.push_str(&format!("/{}", record));
                        client_builder
                            .local_address(IpAddr::from_str("0.0.0.0")?)
                            .build()?
                    }
                    Record::AAAA => {
                        request_url.push_str(&format!("/{}", record));
                        client_builder
                            .local_address(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)))
                            .build()?
                    }
                };

                for m in &self.methods {
                    match m {
                        Method::POST => {
                            let resp = client
                                .post(&request_url)
                                .basic_auth(
                                    &self.credentials.username,
                                    Some(&self.credentials.password),
                                )
                                .send()
                                .await?;
                            let text = resp.text().await?;
                            API::print_response(&request_url, m, &text);
                        }
                        Method::DELETE => {
                            let resp = client
                                .delete(&request_url)
                                .basic_auth(
                                    &self.credentials.username,
                                    Some(&self.credentials.password),
                                )
                                .send()
                                .await?;
                            let text = resp.text().await?;
                            API::print_response(&request_url, m, &text);
                        }
                        Method::PUT => {
                            let resp = client
                                .put(&request_url)
                                .basic_auth(
                                    &self.credentials.username,
                                    Some(&self.credentials.password),
                                )
                                .send()
                                .await?;
                            let text = resp.text().await?;
                            API::print_response(&request_url, m, &text);
                        }
                    };
                }
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
                let methods: Vec<&str> = doc["methods"]
                    .as_vec()
                    .expect(&format!(
                        "method list (PUT/POST/DELETE) should be in {file}"
                    ))
                    .iter()
                    .map(|m| m.as_str().expect("should be able to parse methods list"))
                    .collect();
                let records: Vec<&str> = doc["records"]
                    .as_vec()
                    .expect(&format!("records list (A/AAAA) should be in {file}"))
                    .iter()
                    .map(|m| {
                        m.as_str()
                            .expect(&format!("should be able to parse methods list in {file}"))
                    })
                    .collect();
                let api = API::new(
                    doc["url"]
                        .as_str()
                        .expect(&format!("url should be in {file}")),
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

        pub fn from_config_file(filename: &str) -> Vec<API> {
            let yaml = API::load_file(filename);
            API::parse_yaml(yaml, filename)
        }
    }
}

mod ip_checker {
    use std::{
        fs::File,
        io::{Read, Write},
        net::Ipv4Addr,
        str::FromStr,
    };

    const V4_URL: &'static str = "https://api.ipify.org";

    pub struct IP {
        current: Ipv4Addr,
        pub changed: bool,
        filename: String,
    }

    impl IP {
        async fn get_actual_ip() -> Result<String, Box<dyn std::error::Error>> {
            Ok(reqwest::get(V4_URL).await?.text().await?)
        }

        pub async fn compare(&mut self) -> Result<(), Box<dyn std::error::Error>> {
            let actual = IP::get_actual_ip().await?;
            let actual_ip = Ipv4Addr::from_str(&actual)?;
            if actual_ip != self.current {
                self.current = actual_ip;
                self.changed = true;
                self.write_current_ip_to_file()?;
            }
            Ok(())
        }

        fn write_current_ip_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
            let mut file = File::create(&self.filename)?;
            file.write_all(format!("{}", self.current).as_bytes())?;
            Ok(())
        }

        pub fn new(filename: &str) -> IP {
            let ip_result = IP::from_file(filename);
            match ip_result {
                Ok(ip) => ip,
                Err(_) => IP {
                    current: Ipv4Addr::from_str("0.0.0.0")
                        .expect("can create IP from hardcoded str"),
                    changed: false,
                    filename: filename.to_string(),
                },
            }
        }

        pub fn load(filename: String) -> IP {
            let f = filename.as_str();
            IP::new(f)
        }

        fn from_file(file: &str) -> Result<IP, Box<dyn std::error::Error>> {
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
}
