use regex::Regex;
use serde::Deserialize;

/// Represents a log file with an absolute path and an optional code fragment.
#[derive(Debug)]
pub struct LogFile<T: TaskMessage> {
    absolute_path: String,
    code_fragment: Option<CodeFragment<T>>,
}

impl<T: TaskMessage + Deserialize<'static>> RegexParse for LogFile<T> {
    /// Returns the regular expression used to parse a log file.
    fn regex_value() -> regex::Regex {
        regex::Regex::new(r#"^(.+?):(.*)?"#).unwrap()
    }

    /// Creates a new `LogFile` from the given string using regular expression parsing.
    ///
    /// # Arguments
    ///
    /// * `haystack` - A string slice that holds the log line to be parsed.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A `LogFile` instance if parsing is successful, otherwise `None`.
    fn new_from_regex(haystack: &str) -> Option<Self> {
        let cap = &Self::regex_value().captures(haystack)?;
        let absolute_path = cap.get(1).map(|m| m.as_str())?.to_string();
        let haystack = cap.get(2).map(|m| m.as_str())?;

        Some(LogFile {
            absolute_path,
            code_fragment: CodeFragment::new_from_regex(haystack),
        })
    }
}

/// Represents a fragment of code with line and column information, and optional task information.
#[derive(Debug)]
pub struct CodeFragment<T: TaskMessage> {
    line: usize,
    column: usize,
    task_info: Option<Message<T>>,
}

impl<T: TaskMessage + Deserialize<'static>> RegexParse for CodeFragment<T> {
    /// Returns the regular expression used to parse a code fragment.
    fn regex_value() -> regex::Regex {
        regex::Regex::new(r#"(\d+):(\d+):(.*)?"#).unwrap()
    }

    /// Creates a new `CodeFragment` from the given string using regular expression parsing.
    ///
    /// # Arguments
    ///
    /// * `haystack` - A string slice that holds the code fragment to be parsed.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A `CodeFragment` instance if parsing is successful, otherwise `None`.
    fn new_from_regex(haystack: &str) -> Option<Self> {
        let cap = &Self::regex_value().captures(haystack)?;
        let line: usize = cap.get(1).map(|m| m.as_str())?.parse().ok()?;
        let column: usize = cap.get(2).map(|m| m.as_str())?.parse().ok()?;
        let haystack = cap.get(3).map(|m| m.as_str())?;

        Some(CodeFragment {
            line,
            column,
            task_info: Message::new_from_regex(haystack),
        })
    }
}

/// Represents a message with different types of task information.
#[derive(Debug)]
pub enum Message<T: TaskMessage> {
    Warning(T),
}

impl<T: TaskMessage + Deserialize<'static>> RegexParse for Message<T> {
    /// Returns the regular expression used to parse a message.
    fn regex_value() -> regex::Regex {
        regex::Regex::new(r#"\s?(.+?):\s?(.+)"#).unwrap()
    }

    /// Creates a new `Message` from the given string using regular expression parsing.
    ///
    /// # Arguments
    ///
    /// * `haystack` - A string slice that holds the message to be parsed.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A `Message` instance if parsing is successful, otherwise `None`.
    fn new_from_regex(haystack: &str) -> Option<Self> {
        let cap = &Self::regex_value().captures(haystack)?;
        let message_type = cap.get(1).map(|m| m.as_str())?;
        let haystack = cap.get(2).map(|m| m.as_str())?;

        match message_type {
            MessageNames::WARNING => match T::new_from_regex(haystack) {
                Some(value) => Some(Message::Warning(value)),
                None => None,
            },
            _ => None,
        }
    }
}

/// Enum representing the names of message types.
pub enum MessageNames {
    Warning,
}

impl MessageNames {
    const WARNING: &'static str = "warning";
}

/// Represents a warning message with a summary and queue.
#[derive(serde::Deserialize, Debug)]
pub struct MyWarning {
    #[serde(rename = "summary")]
    summary: String,
    #[serde(rename = "queue")]
    queue: String,
}

impl TaskMessage for MyWarning {
    /// Returns the queue of the warning.
    fn task_queue(&self) -> String {
        self.queue.clone()
    }

    /// Returns the summary of the warning.
    fn task_summary(&self) -> String {
        self.summary.clone()
    }

    /// Returns the message to display after the warning is created.
    fn warning_message_after_created(&self) -> String {
        "".to_string()
    }
}

impl RegexParse for MyWarning {
    /// Returns the regular expression used to parse a warning message.
    fn regex_value() -> regex::Regex {
        Regex::new(r#"s#(.+?)#s(.+)?"#).unwrap()
    }

    /// Creates a new `MyWarning` from the given string using regular expression parsing.
    ///
    /// # Arguments
    ///
    /// * `haystack` - A string slice that holds the warning message to be parsed.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A `MyWarning` instance if parsing is successful, otherwise `None`.
    fn new_from_regex(haystack: &str) -> Option<Self> {
        let cap = &Self::regex_value().captures(haystack)?;
        let json_text = cap.get(1).map(|m| m.as_str())?;
        serde_json::from_str(json_text).ok()
    }
}

/// A trait for parsing strings using regular expressions.
pub trait RegexParse: Sized {
    /// Returns the regular expression used for parsing.
    fn regex_value() -> regex::Regex;
    
    /// Creates a new instance from the given string using regular expression parsing.
    ///
    /// # Arguments
    ///
    /// * `haystack` - A string slice that holds the text to be parsed.
    ///
    /// # Returns
    ///
    /// * `Option<Self>` - A new instance if parsing is successful, otherwise `None`.
    fn new_from_regex(haystack: &str) -> Option<Self>;
}

/// A trait representing a task message with methods for retrieving task details.
pub trait TaskMessage: Deserialize<'static> + RegexParse + std::fmt::Debug {
    /// Returns the summary of the task.
    fn task_summary(&self) -> String;
    
    /// Returns the queue of the task.
    fn task_queue(&self) -> String;
    
    /// Returns the message to display after the task is created.
    fn warning_message_after_created(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests parsing a log file where the queue field is missing from the JSON.
    #[test]
    fn test_log_file_parse_missing_queue() {
        let log_line = r#"path/to/file.log:123:456: warning: s#{"summary": "Create a task"}#s"#;
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => match log_file.code_fragment {
                Some(code_fragment) => match code_fragment.task_info {
                    Some(_) => panic!(
                        "Expected parsing to fail due to missing 'queue' field"
                    ),
                    None => return,
                },
                None => panic!(
                    "Expected code fragment to be found as '123:456' is present"
                ),
            },
            None => panic!(
                "Expected log file to be found as 'path/to/file.log' is present"
            ),
        }
    }

    /// Tests parsing a log file where the s##s delimiters are missing from the JSON.
    #[test]
    fn test_log_file_parse_missing_s_s_delimiters() {
        let log_line = r#"path/to/file.log:123:456: warning: {"queue": "TESTAPI", "summary": "Create a task"}"#;
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => match log_file.code_fragment {
                Some(code_fragment) => match code_fragment.task_info {
                    Some(_) => {
                        panic!("Expected parsing to fail due to missing 's##s' delimiters")
                    }
                    None => return,
                },
                None => panic!(
                    "Expected code fragment to be found as '123:456' is present"
                ),
            },
            None => panic!(
                "Expected log file to be found as 'path/to/file.log' is present"
            ),
        }
    }

    /// Tests parsing a log file where the warning keyword is missing.
    #[test]
    fn test_log_file_parse_missing_warning_keyword() {
        let log_line =
            r#"path/to/file.log:123:456: s#{"queue": "TESTAPI", "summary": "Create a task"}#s"#;
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => match log_file.code_fragment {
                Some(code_fragment) => match code_fragment.task_info {
                    Some(_) => panic!(
                        "Expected parsing to fail due to missing 'warning' keyword"
                    ),
                    None => return,
                },
                None => panic!("Expected code fragment to be found as '123:456' is present"),
            },
            None => panic!("Expected log file to be found as 'path/to/file.log' is present"),
        }
    }

    /// Tests parsing a log file where the warning formatting is incorrect.
    #[test]
    fn test_log_file_parse_incorrect_warning_formatting() {
        let log_line = r#"path/to/file.log:123:456:warning:s#{"queue": "TESTAPI", "summary": "Create a task"}#s"#;
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => match log_file.code_fragment {
                Some(code_fragment) => match code_fragment.task_info {
                    Some(_) => return,
                    None => panic!(
                        "Expected parsing to succeed as whitespace around ':' is optional"
                    ),
                },
                None => panic!(
                    "Expected code fragment to be found as '123:456' is present"
                ),
            },
            None => panic!(
                "Expected log file to be found as 'path/to/file.log' is present"
            ),
        }
    }

    /// Tests parsing a log file with an incorrect format.
    #[test]
    fn test_log_file_parse_incorrect_format() {
        let log_line = r#"invalid format"#;
        let log_file = LogFile::<MyWarning>::new_from_regex(log_line);
        assert!(
            log_file.is_none(),
            "Expected parsing to fail due to incorrect log format"
        );
    }

    /// Tests parsing a log file where s##s delimiters are missing.
    #[test]
    fn test_log_file_parse_success_without_s_s() {
        let log_line = "path/to/file.log:123:456:warning: some message";
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => {
                match log_file.code_fragment {
                    Some(code_fragment) => match code_fragment.task_info {
                        Some(_) => {
                            panic!("Expected parsing to fail due to missing 's##s' delimiters")
                        }
                        None => return,
                    },
                    None => panic!(
                    "Expected code fragment to be found as '123:456' is present"
                ),
                }
            }
            None => panic!(
                "Expected log file to be found as 'path/to/file.log' is present"
            ),
        }
    }

    /// Tests parsing a log file where s##s delimiters are empty.
    #[test]
    fn test_log_file_parse_success_with_empty_s_s() {
        let log_line = "path/to/file.log:123:456:warning: s##s";
        match LogFile::<MyWarning>::new_from_regex(log_line) {
            Some(log_file) => match log_file.code_fragment {
                Some(code_fragment) => match code_fragment.task_info {
                    Some(_) => panic!("Expected parsing to fail due to missing JSON inside 's##s'"),
                    None => return,
                },
                None => panic!("Expected code fragment to be found as '123:456' is present")
            },
            None => panic!("Expected log file to be found as 'path/to/file.log' is present")
        }
    }

    /// Tests successful parsing of a valid log file.
    #[test]
    fn test_log_file_parse_success_valid_log() {
        let log_line = r#"path/to/file.log:123:456: warning: s#{"queue": "TESTAPI", "summary": "Create a task"}#s"#;
        let log_file = LogFile::<MyWarning>::new_from_regex(log_line).unwrap(); // Expecting a successful parse

        assert_eq!(log_file.absolute_path, "path/to/file.log");

        // Assuming CodeFragment::new_from_regex returns a Some value
        let code_fragment = log_file.code_fragment.unwrap();
        assert_eq!(code_fragment.line, 123);
        assert_eq!(code_fragment.column, 456);

        // Assuming Message::new_from_regex returns a Some value
        let message = code_fragment.task_info.unwrap();
        match message {
            Message::Warning(warning) => {
                assert_eq!(warning.queue, "TESTAPI");
                assert_eq!(warning.summary, "Create a task");
                // Add assertions for warning content if needed
            }
        }
    }

    /// Tests parsing a log file with an invalid format.
    #[test]
    fn test_log_file_parse_failure_invalid_format() {
        let invalid_log_line = "invalid format";
        let log_file = LogFile::<MyWarning>::new_from_regex(invalid_log_line);
        assert!(log_file.is_none());
    }
}
