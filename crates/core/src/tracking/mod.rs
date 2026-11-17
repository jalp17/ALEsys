//! Experiment tracking via MLflow (Fase I+D)
//!
//! Lightweight client for MLflow Tracking Server REST API.
//! Activated when MLFLOW_TRACKING_URI env var is set.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    pub tracking_uri: Option<String>,
    pub experiment_name: String,
}

impl TrackingConfig {
    pub fn from_env() -> Self {
        Self {
            tracking_uri: std::env::var("MLFLOW_TRACKING_URI").ok(),
            experiment_name: std::env::var("MLFLOW_EXPERIMENT").unwrap_or_else(|_| "alesys-ingestion".to_string()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.tracking_uri.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct MlflowClient {
    pub config: TrackingConfig,
    http: reqwest::Client,
}

impl MlflowClient {
    pub fn new(config: TrackingConfig) -> Self {
        Self { config, http: reqwest::Client::new() }
    }

    pub async fn create_run(&self, run_name: &str) -> Result<String, String> {
        let uri = self.config.tracking_uri.as_ref().ok_or("tracking disabled")?;
        // Get or create experiment
        let exp_id = self.get_or_create_experiment(&uri, &self.config.experiment_name).await?;
        let body = serde_json::json!({
            "experiment_id": exp_id,
            "start_time": now_ms(),
            "tags": [{"key": "mlflow.runName", "value": run_name}],
        });
        let resp = self.http.post(format!("{}/api/2.0/mlflow/runs/create", uri))
            .json(&body).send().await.map_err(|e| e.to_string())?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        json.get("run").and_then(|r| r.get("info")).and_then(|i| i.get("run_id"))
            .and_then(|v| v.as_str()).map(|s| s.to_string())
            .ok_or_else(|| "no run_id".to_string())
    }

    pub async fn log_param(&self, run_id: &str, key: &str, value: &str) -> Result<(), String> {
        let uri = self.config.tracking_uri.as_ref().ok_or("tracking disabled")?;
        let body = serde_json::json!({"run_id": run_id, "key": key, "value": value});
        self.http.post(format!("{}/api/2.0/mlflow/runs/log-parameter", uri))
            .json(&body).send().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn log_metric(&self, run_id: &str, key: &str, value: f64) -> Result<(), String> {
        let uri = self.config.tracking_uri.as_ref().ok_or("tracking disabled")?;
        let body = serde_json::json!({
            "run_id": run_id,
            "key": key,
            "value": value,
            "timestamp": now_ms(),
            "step": 0,
        });
        self.http.post(format!("{}/api/2.0/mlflow/runs/log-metric", uri))
            .json(&body).send().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn end_run(&self, run_id: &str) -> Result<(), String> {
        let uri = self.config.tracking_uri.as_ref().ok_or("tracking disabled")?;
        let body = serde_json::json!({"run_id": run_id, "end_time": now_ms(), "status": "FINISHED"});
        self.http.post(format!("{}/api/2.0/mlflow/runs/update", uri))
            .json(&body).send().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_or_create_experiment(&self, uri: &str, name: &str) -> Result<String, String> {
        // Try get
        let resp = self.http.get(format!("{}/api/2.0/mlflow/experiments/get-by-name?experiment_name={}", uri, name))
            .send().await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(id) = json.get("experiment").and_then(|e| e.get("experiment_id")).and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }
        // Create
        let body = serde_json::json!({"name": name});
        let resp = self.http.post(format!("{}/api/2.0/mlflow/experiments/create", uri))
            .json(&body).send().await.map_err(|e| e.to_string())?;
        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        json.get("experiment_id").and_then(|v| v.as_str()).map(|s| s.to_string())
            .ok_or_else(|| "no experiment_id".to_string())
    }
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64
}
