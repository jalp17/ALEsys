use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        let cleaned = s.trim_matches(|c| c == '[' || c == ']' || c == ' ');
        match cleaned.to_uppercase().as_str() {
            "ERROR" | "ERR" | "FATAL" | "PANIC" => LogLevel::Error,
            "WARN" | "WARNING" => LogLevel::Warning,
            "INFO" => LogLevel::Info,
            "DEBUG" | "DBG" => LogLevel::Debug,
            "TRACE" | "TRC" => LogLevel::Trace,
            _ => LogLevel::Unknown,
        }
    }
}

pub struct LogParser;

impl LogParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_line(&self, line: &str) -> LogEntry {
        let (timestamp, rest) = self.extract_timestamp(line);
        let (level, rest) = self.extract_level(rest);
        let (source, message) = self.extract_source_and_message(rest);

        LogEntry {
            timestamp,
            level,
            source: source.to_string(),
            message: message.to_string(),
            stack_trace: None,
            raw: line.to_string(),
        }
    }

    pub fn parse_logs(&self, input: &str) -> Vec<LogEntry> {
        input.lines().map(|line| self.parse_line(line)).collect()
    }

    pub fn extract_errors(&self, input: &str) -> Vec<LogEntry> {
        self.parse_logs(input)
            .into_iter()
            .filter(|e| e.level == LogLevel::Error)
            .collect()
    }

    fn extract_timestamp<'a>(&self, line: &'a str) -> (Option<String>, &'a str) {
        let trimmed = line.trim_start();
        if trimmed.len() >= 19 {
            let maybe_ts = &trimmed[..19];
            if maybe_ts.contains('-') && maybe_ts.contains(':')
                && (maybe_ts.contains('T') || maybe_ts.contains(' '))
            {
                return (Some(maybe_ts.to_string()), trimmed[19..].trim_start());
            }
        }
        (None, trimmed)
    }

    fn extract_level<'a>(&self, rest: &'a str) -> (LogLevel, &'a str) {
        for prefix in &["[ERROR]", "[WARN]", "[INFO]", "[DEBUG]", "[TRACE]",
                         "ERROR:", "WARN:", "INFO:", "DEBUG:", "TRACE:",
                         "error]", "warn]", "info]", "debug]", "trace]"] {
            if let Some(pos) = rest.find(prefix) {
                let after = rest[pos + prefix.len()..].trim_start();
                let level = LogLevel::from_str(&rest[pos..pos + prefix.len()]);
                return (level, after);
            }
        }
        (LogLevel::Unknown, rest)
    }

    fn extract_source_and_message<'a>(&self, rest: &'a str) -> (&'a str, &'a str) {
        if let Some(pos) = rest.find("]: ") {
            (&rest[..pos], rest[pos + 3..].trim_start())
        } else if let Some(pos) = rest.find(" - ") {
            (&rest[..pos], rest[pos + 3..].trim_start())
        } else if let Some(pos) = rest.find(": ") {
            (&rest[..pos], rest[pos + 2..].trim_start())
        } else {
            ("unknown", rest)
        }
    }
}

impl Default for LogParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_line() {
        let parser = LogParser::new();
        let entry = parser.parse_line("2024-01-15T10:30:00 [ERROR] main: Connection refused");
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.source, "main");
        assert!(entry.message.contains("Connection refused"));
    }

    #[test]
    fn test_parse_log_level_variants() {
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("ERR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("FATAL"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("PANIC"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warning);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("TRACE"), LogLevel::Trace);
    }

    #[test]
    fn test_extract_errors() {
        let parser = LogParser::new();
        let logs = "[ERROR] db: timeout\n[INFO] server: started\n[ERROR] net: reset";
        let errors = parser.extract_errors(logs);
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn test_parse_multiple_lines() {
        let parser = LogParser::new();
        let input = "[INFO] app: starting\n[WARN] config: using defaults\n[ERROR] db: connection failed";
        let entries = parser.parse_logs(input);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].level, LogLevel::Info);
        assert_eq!(entries[1].level, LogLevel::Warning);
        assert_eq!(entries[2].level, LogLevel::Error);
    }
}
