use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Manual,
    Cron,
    Webhook,
    Event,
    Schedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub trigger_type: TriggerType,
    pub schedule: Option<String>,
    pub event_name: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub config: TriggerConfig,
    pub enabled: bool,
}

impl Trigger {
    pub fn new(id: &str, name: &str, config: TriggerConfig) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            config,
            enabled: true,
        }
    }

    pub fn should_fire(&self, context: &TriggerContext) -> bool {
        if !self.enabled {
            return false;
        }

        match &self.config.trigger_type {
            TriggerType::Manual => context.manual_trigger,
            TriggerType::Cron => {
                if let Some(schedule) = &self.config.schedule {
                    context.current_minute == *schedule
                } else {
                    false
                }
            }
            TriggerType::Webhook => context.webhook_received,
            TriggerType::Event => {
                if let Some(event) = &self.config.event_name {
                    context.event_name.as_ref() == Some(event)
                } else {
                    false
                }
            }
            TriggerType::Schedule => context.scheduled_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerContext {
    pub manual_trigger: bool,
    pub current_minute: String,
    pub webhook_received: bool,
    pub event_name: Option<String>,
    pub scheduled_time: bool,
}

impl Default for TriggerContext {
    fn default() -> Self {
        Self {
            manual_trigger: false,
            current_minute: String::new(),
            webhook_received: false,
            event_name: None,
            scheduled_time: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_trigger() {
        let trigger = Trigger::new("t1", "Manual", TriggerConfig {
            trigger_type: TriggerType::Manual,
            schedule: None,
            event_name: None,
            webhook_url: None,
        });
        let ctx = TriggerContext { manual_trigger: true, ..Default::default() };
        assert!(trigger.should_fire(&ctx));
    }

    #[test]
    fn test_disabled_trigger() {
        let mut trigger = Trigger::new("t1", "Manual", TriggerConfig {
            trigger_type: TriggerType::Manual,
            schedule: None,
            event_name: None,
            webhook_url: None,
        });
        trigger.enabled = false;
        let ctx = TriggerContext { manual_trigger: true, ..Default::default() };
        assert!(!trigger.should_fire(&ctx));
    }

    #[test]
    fn test_event_trigger() {
        let trigger = Trigger::new("t1", "OnDeploy", TriggerConfig {
            trigger_type: TriggerType::Event,
            schedule: None,
            event_name: Some("deploy".to_string()),
            webhook_url: None,
        });
        let ctx = TriggerContext { event_name: Some("deploy".to_string()), ..Default::default() };
        assert!(trigger.should_fire(&ctx));
    }

    #[test]
    fn test_webhook_trigger() {
        let trigger = Trigger::new("t1", "Webhook", TriggerConfig {
            trigger_type: TriggerType::Webhook,
            schedule: None,
            event_name: None,
            webhook_url: Some("http://localhost/hook".to_string()),
        });
        let ctx = TriggerContext { webhook_received: true, ..Default::default() };
        assert!(trigger.should_fire(&ctx));
    }

    #[test]
    fn test_cron_trigger() {
        let trigger = Trigger::new("t1", "Every Minute", TriggerConfig {
            trigger_type: TriggerType::Cron,
            schedule: Some("*/5".to_string()),
            event_name: None,
            webhook_url: None,
        });
        let ctx = TriggerContext { current_minute: "*/5".to_string(), ..Default::default() };
        assert!(trigger.should_fire(&ctx));
    }
}