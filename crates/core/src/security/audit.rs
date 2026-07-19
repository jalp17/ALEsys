use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub level: AuditLevel,
    pub action: String,
    pub resource: String,
    pub user: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

pub struct AuditLog {
    events: Vec<AuditEvent>,
    max_events: usize,
}

impl AuditLog {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    pub fn log(&mut self, level: AuditLevel, action: &str, resource: &str, user: &str) {
        let event = AuditEvent {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level,
            action: action.to_string(),
            resource: resource.to_string(),
            user: user.to_string(),
            details: None,
            ip_address: None,
        };

        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }

        self.events.push(event);
    }

    pub fn log_with_details(&mut self, level: AuditLevel, action: &str, resource: &str, user: &str, details: &str, ip: &str) {
        let event = AuditEvent {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level,
            action: action.to_string(),
            resource: resource.to_string(),
            user: user.to_string(),
            details: Some(details.to_string()),
            ip_address: Some(ip.to_string()),
        };

        if self.events.len() >= self.max_events {
            self.events.remove(0);
        }

        self.events.push(event);
    }

    pub fn query(&self, level: Option<AuditLevel>) -> Vec<&AuditEvent> {
        self.events.iter()
            .filter(|e| match &level {
                Some(l) => std::mem::discriminant(&e.level) == std::mem::discriminant(l),
                None => true,
            })
            .collect()
    }

    pub fn stats(&self) -> AuditStats {
        let mut info = 0;
        let mut warning = 0;
        let mut error = 0;
        let mut critical = 0;

        for event in &self.events {
            match event.level {
                AuditLevel::Info => info += 1,
                AuditLevel::Warning => warning += 1,
                AuditLevel::Error => error += 1,
                AuditLevel::Critical => critical += 1,
            }
        }

        AuditStats { total: self.events.len(), info, warning, error, critical }
    }
}

#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total: usize,
    pub info: usize,
    pub warning: usize,
    pub error: usize,
    pub critical: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log() {
        let mut log = AuditLog::new(100);
        log.log(AuditLevel::Info, "login", "/auth", "user1");
        assert_eq!(log.stats().total, 1);
    }

    #[test]
    fn test_audit_max_events() {
        let mut log = AuditLog::new(2);
        log.log(AuditLevel::Info, "a", "/r", "u");
        log.log(AuditLevel::Info, "b", "/r", "u");
        log.log(AuditLevel::Info, "c", "/r", "u");
        assert_eq!(log.stats().total, 2);
    }

    #[test]
    fn test_audit_query() {
        let mut log = AuditLog::new(100);
        log.log(AuditLevel::Info, "a", "/r", "u");
        log.log(AuditLevel::Error, "b", "/r", "u");
        let errors = log.query(Some(AuditLevel::Error));
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_audit_stats() {
        let mut log = AuditLog::new(100);
        log.log(AuditLevel::Info, "a", "/r", "u");
        log.log(AuditLevel::Warning, "b", "/r", "u");
        log.log(AuditLevel::Error, "c", "/r", "u");
        let stats = log.stats();
        assert_eq!(stats.info, 1);
        assert_eq!(stats.warning, 1);
        assert_eq!(stats.error, 1);
    }
}