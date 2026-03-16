use crate::time_tools;

/// Represents the log level.
///
/// Log levels are used to indicate the severity of a log message.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl From<String> for LogLevel {
    /// Parses the given string into a `LogLevel` enum variant.
    ///
    /// # Arguments
    ///
    /// * `string` - A string representing the log level.
    ///
    /// # Returns
    ///
    /// Returns a `LogLevel` enum variant corresponding to the given string (after transforming to uppercase):
    /// - If the string is "DEBUG", returns `LogLevel::Debug`.
    /// - If the string is "WARNING", returns `LogLevel::Warning`.
    /// - If the string is "ERROR", returns `LogLevel::Error`.
    /// - For any other string, returns `LogLevel::Info`.
    fn from(string: String) -> LogLevel {
        match string.to_uppercase().as_str() {
            "DEBUG" => LogLevel::Debug,
            "WARNING" => LogLevel::Warning,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}


/// A Logger struct stores the level of logging
///
/// # Example
///
/// ```
/// let logger = Logger {
///     level: LogLevel::Info,
/// };
/// println!("{:?}", logger);
/// ```
#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

#[allow(dead_code)]
impl Logger {
    /// Create a new instance of `Logger`.
    ///
    /// The `new` function will read the `DDNS_LOG_LEVEL` environment variable
    /// and create a new instance with the obtained log level. If the environment
    /// variable is not found, the default log level will be `INFO`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logging::Logger;
    ///
    /// let logger = Logger::new();
    /// ```
    pub fn new() -> Self {
        let level = match std::env::var("DDNS_LOG_LEVEL") {
            Ok(val) => val,
            Err(_) => "INFO".to_string(),
        };
        Self {
            level: level.into(),
        }
    }
    /// Prints a log message with a specified log level and the current time.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level of the message.
    /// * `message` - The log message to be printed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let logger = Logger::new();
    /// logger.print_log("INFO", "This is an info message");
    /// ```
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
    /// Prints an info log message if the log level is set to `Info`.
    ///
    /// # Arguments
    ///
    /// * `message` - A string slice representing the log message.
    ///
    /// # Examples
    ///
    /// ```
    /// let logger = Logger::new(LogLevel::Info);
    ///
    /// logger.info("This is an info message");
    /// ```
    pub fn info(&self, message: &str) {
        if self.level <= LogLevel::Info {
            self.print_log("INFO", message)
        }
    }
    /// Logs a debug message with the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - The debug message to be logged.
    ///
    /// # Example
    ///
    /// ```
    /// let logger = Logger::new(LogLevel::Debug);
    /// logger.debug("This is a debug message");
    /// ```
    pub fn debug(&self, message: &str) {
        if self.level <= LogLevel::Debug {
            self.print_log("DEBUG", message)
        }
    }
    /// Prints a warning log message if the log level is equal to or greater than `LogLevel::Warning`.
    ///
    /// # Arguments
    ///
    /// * `message` - The warning log message to be printed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use log::{LogLevel, Logger};
    ///
    /// let logger = Logger::new(LogLevel::Warning);
    /// logger.warning("This is a warning message");
    /// ```
    pub fn warning(&self, message: &str) {
       if self.level <= LogLevel::Warning {
           self.print_log("WARNING", message)
       }
    }
    /// Logs an error message if the current logging level is equal to or higher than `LogLevel::Error`.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to be logged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use my_logging::Logger;
    ///
    /// let logger = Logger::new();
    /// logger.error("An error occurred!");
    /// ```
    pub fn error(&self, message: &str) {
        if self.level <= LogLevel::Error {
            let newline = if message.ends_with("\n") { "" } else { "\n" };
            eprint!(
                "{} |ERROR| {}{}",
                time_tools::now_as_string(),
                message,
                newline
            );
        }
    }
}
