//! Verification of LLM responses: faithfulness, hallucination, numeric consistency, citations.
//!
//! Wraps scripts/verify_response.py.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub faithfulness: f64,
    pub hallucination_score: f64,
    pub numeric_consistency: f64,
    pub citations_valid: bool,
    pub overall_pass: bool,
    pub details: serde_json::Value,
}

pub struct Verifier {
    pub script_path: PathBuf,
    pub python: String,
}

impl Default for Verifier {
    fn default() -> Self {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let script = std::path::PathBuf::from(manifest)
            .join("../../scripts/verify_response.py");
        Self {
            script_path: script,
            python: "python3".to_string(),
        }
    }
}

impl Verifier {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path, python: "python3".to_string() }
    }

    pub fn verify(
        &self,
        question: &str,
        answer: &str,
        contexts: &[String],
        citations: &[serde_json::Value],
    ) -> Result<VerificationResult, String> {
        let input = serde_json::json!({
            "question": question,
            "answer": answer,
            "contexts": contexts,
            "citations": citations,
        });
        let mut child = Command::new(&self.python)
            .arg(&self.script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn verifier: {}", e))?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input.to_string().as_bytes()).map_err(|e| e.to_string())?;
        }
        let output = child.wait_with_output().map_err(|e| format!("Verifier error: {}", e))?;
        if !output.status.success() {
            return Err(format!("Verifier failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout).map_err(|e| format!("Parse error: {} - {}", e, stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let v = Verifier::default();
        assert_eq!(v.python, "python3");
    }

    #[test]
    fn test_verify_heuristic() {
        let v = Verifier::default();
        let ctx = vec!["The sky is blue.".to_string()];
        let res = v.verify("What color?", "The sky is blue.", &ctx, &[]).unwrap();
        assert!(res.faithfulness > 0.0);
        assert!(res.overall_pass);
    }
}
