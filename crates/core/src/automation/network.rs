use crate::executor::{self, ExecutorConfig};

pub async fn check_url(url: &str) -> Result<String, String> {
    let config = ExecutorConfig {
        timeout_ms: 15_000,
        ..Default::default()
    };

    let result = executor::execute("curl", &["-s", "-o", "/dev/null", "-w", "%{http_code}", url], None, &config).await?;

    if result.exit_code == 0 {
        Ok(format!("URL {} responded with HTTP {}", url, result.stdout.trim()))
    } else {
        Err(format!("Failed to reach {}: {}", url, result.stderr))
    }
}

pub async fn ping(host: &str, count: u32) -> Result<String, String> {
    let config = ExecutorConfig {
        timeout_ms: 30_000,
        ..Default::default()
    };

    let result = executor::execute("ping", &["-c", &count.to_string(), host], None, &config).await?;

    if result.exit_code == 0 {
        Ok(result.stdout)
    } else {
        Err(format!("Ping to {} failed:\n{}", host, result.stderr))
    }
}
