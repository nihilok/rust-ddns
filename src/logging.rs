use crate::time_tools;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
}

impl From<String> for LogLevel {
    fn from(string: String) -> LogLevel {
        match string.to_uppercase().as_str() {
            "DEBUG" => LogLevel::DEBUG,
            "WARNING" => LogLevel::WARNING,
            "ERROR" => LogLevel::ERROR,
            _ => LogLevel::INFO,
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

#[allow(dead_code)]
impl Logger {
    pub fn new() -> Self {
        let level = match std::env::var("DDNS_LOG_LEVEL") {
            Ok(val) => val,
            Err(_) => "INFO".to_string(),
        };
        Self {
            level: level.into(),
        }
    }
    fn print_log(&self, level: &str, message: &str) {
        let newline = if message.ends_with("\n") { "" } else { "\n" };
        print!(
            "{} |{}| {}{}",
            time_tools::now_as_string(),
            level,
            message,
            newline
        );
    }
    pub fn info(&self, message: &str) {
        if self.level <= LogLevel::INFO {
            self.print_log("INFO", message)
        }
    }
    pub fn debug(&self, message: &str) {
        if self.level <= LogLevel::DEBUG {
            self.print_log("DEBUG", message)
        }
    }
    pub fn warning(&self, message: &str) {
       if self.level <= LogLevel::WARNING {
           self.print_log("WARNING", message)
       }
    }
    pub fn error(&self, message: &str) {
        if self.level <= LogLevel::ERROR {
            let newline = if message.ends_with("\n") { "" } else { "\n" };
            eprint!(
                "{} |{}| {}{}",
                time_tools::now_as_string(),
                "ERROR",
                message,
                newline
            );
        }
    }
}
