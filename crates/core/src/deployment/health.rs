use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub uptime_secs: u64,
    pub version: String,
}

impl HealthCheck {
    pub fn new(version: &str, uptime_secs: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            components: Vec::new(),
            uptime_secs,
            version: version.to_string(),
        }
    }

    pub fn add_component(&mut self, component: ComponentHealth) {
        if component.status == HealthStatus::Unhealthy {
            self.status = HealthStatus::Unhealthy;
        } else if component.status == HealthStatus::Degraded && self.status == HealthStatus::Healthy {
            self.status = HealthStatus::Degraded;
        }
        self.components.push(component);
    }

    pub fn check_database(&mut self, connected: bool, latency_ms: u64) {
        let status = if connected { HealthStatus::Healthy } else { HealthStatus::Unhealthy };
        self.add_component(ComponentHealth {
            name: "database".to_string(),
            status,
            message: if connected { Some("Connected".to_string()) } else { Some("Disconnected".to_string()) },
            latency_ms: Some(latency_ms),
        });
    }

    pub fn check_memory(&mut self, used_mb: u64, total_mb: u64) {
        let ratio = used_mb as f64 / total_mb as f64;
        let status = if ratio < 0.8 {
            HealthStatus::Healthy
        } else if ratio < 0.95 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };

        self.add_component(ComponentHealth {
            name: "memory".to_string(),
            status,
            message: Some(format!("{}/{}MB", used_mb, total_mb)),
            latency_ms: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_healthy() {
        let mut health = HealthCheck::new("1.0.0", 100);
        health.check_database(true, 5);
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_unhealthy() {
        let mut health = HealthCheck::new("1.0.0", 100);
        health.check_database(false, 0);
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_memory_degraded() {
        let mut health = HealthCheck::new("1.0.0", 100);
        health.check_memory(850, 1000);
        assert_eq!(health.status, HealthStatus::Degraded);
    }
}