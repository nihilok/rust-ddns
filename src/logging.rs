use crate::{time_tools, LOG_LEVEL};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
}

#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new() -> Self {
        Self { level: LOG_LEVEL }
    }
    fn print_log(&self, level: &str, message: &str) {
        let newline = if message.ends_with("\n") { "" } else { "\n" };
        print!("{} |{}| {}{}", time_tools::now_as_string(), level, message, newline);
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
            eprint!("{} |{}| {}{}", time_tools::now_as_string(), "ERROR", message, newline);
        }
    }
}
