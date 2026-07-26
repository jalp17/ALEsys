use crate::ingestion::{
    IngestionConfig, IngestionMode, IngestionPlugin, IngestionProgress, IngestionStage,
};
use crate::plugin::{Plugin, PluginContext, PluginMetadata, PluginPermission, PluginResult};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

fn make_context() -> PluginContext {
    PluginContext {
        work_dir: PathBuf::from("/tmp"),
        allowed_paths: vec!["/tmp".to_string()],
        config: HashMap::new(),
        request_id: "test-req-1".to_string(),
    }
}

fn make_config() -> IngestionConfig {
    IngestionConfig {
        model_dir: PathBuf::from("/tmp/test-models"),
        output_base_dir: PathBuf::from("/tmp/test-output"),
        fallback_enabled: true,
        default_ocr_langs: vec!["en".to_string()],
        max_parallel: 1,
        timeout_hours: 1,
    }
}

#[test]
fn test_plugin_creation() {
    let plugin = IngestionPlugin::new(make_config());
    assert!(plugin.processor.is_none());
}

#[test]
fn test_plugin_metadata() {
    let plugin = IngestionPlugin::new(make_config());
    let meta = plugin.metadata();
    assert_eq!(meta.id, "ingestion");
    assert_eq!(meta.name, "PDF Ingestion Plugin");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.author, "ALEsys");
    assert!(meta
        .permissions
        .contains(&PluginPermission::FilesystemRead {
            allowed_paths: vec!["/tmp".to_string(), "/data".to_string()],
        }));
    assert!(meta
        .permissions
        .contains(&PluginPermission::Execute {
            allowed_commands: vec!["python3".to_string(), "magic-pdf".to_string()],
        }));
}

#[test]
fn test_plugin_commands() {
    let plugin = IngestionPlugin::new(make_config());
    assert!(plugin.can_handle("ingest.pdf"));
    assert!(plugin.can_handle("ingest.batch"));
    assert!(!plugin.can_handle("unknown"));
    assert_eq!(
        plugin.supported_commands(),
        vec!["ingest.pdf".to_string(), "ingest.batch".to_string()]
    );
}

#[test]
fn test_plugin_init_creates_processor() {
    let mut plugin = IngestionPlugin::new(make_config());
    let ctx = make_context();
    let result = plugin.init(&ctx);
    assert!(result.is_ok());
    assert!(plugin.processor.is_some());
}

#[test]
fn test_plugin_execute_ingest_pdf_missing_arg() {
    let mut plugin = IngestionPlugin::new(make_config());
    let _ = plugin.init(&make_context());
    let ctx = make_context();
    let result = plugin.execute("ingest.pdf", &[], &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Missing pdf_path"));
}

#[test]
fn test_plugin_execute_ingest_pdf_fails_without_magic_pdf() {
    let mut plugin = IngestionPlugin::new(make_config());
    let _ = plugin.init(&make_context());
    let ctx = make_context();
    let args = vec!["/tmp/does-not-exist.pdf".to_string()];
    let result = plugin.execute("ingest.pdf", &args, &ctx);
    assert!(result.is_err());
}

#[test]
fn test_plugin_execute_unknown_command() {
    let mut plugin = IngestionPlugin::new(make_config());
    let _ = plugin.init(&make_context());
    let ctx = make_context();
    let result = plugin.execute("unknown.cmd", &[], &ctx);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown command"));
}

#[test]
fn test_plugin_shutdown() {
    let mut plugin = IngestionPlugin::new(make_config());
    let ctx = make_context();
    let _ = plugin.init(&ctx);
    assert!(plugin.processor.is_some());
    let result = plugin.shutdown();
    assert!(result.is_ok());
    assert!(plugin.processor.is_none());
}
