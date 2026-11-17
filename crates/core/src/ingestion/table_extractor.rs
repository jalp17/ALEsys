//! Table extractor: PDF -> structured tables (CSV/Parquet)
//!
//! Wraps scripts/extract_tables.py (PyMuPDF + Camelot).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    pub page: u32,
    pub bbox: Option<Vec<f32>>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub method: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExtractionResult {
    pub tables: Vec<ExtractedTable>,
    pub count: usize,
}

pub struct TableExtractor {
    pub script_path: PathBuf,
    pub python: String,
}

impl Default for TableExtractor {
    fn default() -> Self {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let script = std::path::PathBuf::from(manifest)
            .join("../../scripts/extract_tables.py");
        Self {
            script_path: script,
            python: "python3".to_string(),
        }
    }
}

impl TableExtractor {
    pub fn new(script_path: PathBuf) -> Self {
        Self { script_path, python: "python3".to_string() }
    }

    pub fn extract(&self, pdf_path: &Path, output_dir: Option<&Path>) -> Result<TableExtractionResult, String> {
        let mut cmd = Command::new(&self.python);
        cmd.arg(&self.script_path).arg(pdf_path);
        if let Some(out) = output_dir {
            cmd.arg("--output").arg(out);
        }
        let output = cmd.output().map_err(|e| format!("Failed to run table extractor: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "Table extractor failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse output: {} - {}", e, stdout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let te = TableExtractor::default();
        assert_eq!(te.python, "python3");
    }

    #[test]
    fn test_parse_empty() {
        let json = r#"{"tables":[],"count":0}"#;
        let r: TableExtractionResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.count, 0);
    }
}
