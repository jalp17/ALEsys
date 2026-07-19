use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_updater::UpdaterExt;
use serde::{Deserialize, Serialize};

use alesys_core::executor::{self, ExecutorConfig};
use alesys_core::fs_ops;
use alesys_core::automation::system;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
    pub memory_total: u64,
    pub memory_available: u64,
    pub cpu_count: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecuteResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Deserialize, Debug)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub filters: Option<Vec<FileFilter>>,
    #[allow(dead_code)]
    pub multiple: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[command]
pub async fn open_file_dialog(
    app: AppHandle,
    options: Option<FileDialogOptions>,
) -> Result<Option<Vec<String>>, String> {
    let mut dialog = app.dialog().file();

    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(filters) = opts.filters {
            for filter in filters {
                dialog = dialog.add_filter(filter.name, &filter.extensions.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
        }
        if opts.multiple.unwrap_or(false) {
            let result = dialog.blocking_pick_files();
            return Ok(result.map(|v| v.into_iter().map(|p| p.to_string()).collect()));
        }
    }

    let result = dialog.blocking_pick_file();
    Ok(result.map(|p| vec![p.to_string()]))
}

#[command]
pub async fn open_folder_dialog(
    app: AppHandle,
    options: Option<FileDialogOptions>,
) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file();

    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
    }

    let result = dialog.blocking_pick_folder();
    Ok(result.map(|p| p.to_string()))
}

#[command]
pub async fn save_file_dialog(
    app: AppHandle,
    options: Option<FileDialogOptions>,
) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file();

    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(filters) = opts.filters {
            for filter in filters {
                dialog = dialog.add_filter(filter.name, &filter.extensions.iter().map(|s| s.as_str()).collect::<Vec<_>>());
            }
        }
    }

    let result = dialog.blocking_save_file();
    Ok(result.map(|p| p.to_string()))
}

#[command]
pub async fn read_file(path: String) -> Result<String, String> {
    fs_ops::read_file(&path).await
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    fs_ops::write_file(&path, &content).await
}

#[command]
pub async fn list_directory(path: String) -> Result<Vec<FileItem>, String> {
    let items = fs_ops::list_directory(&path).await?;
    Ok(items.into_iter().map(|i| FileItem {
        name: i.name,
        path: i.path,
        is_dir: i.is_dir,
        size: Some(i.size),
        modified: i.modified,
    }).collect())
}

#[command]
pub async fn create_directory(path: String) -> Result<(), String> {
    fs_ops::create_directory(&path).await
}

#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    fs_ops::delete_file(&path).await
}

#[command]
pub async fn copy_file(from: String, to: String) -> Result<(), String> {
    fs_ops::copy_file(&from, &to).await
}

#[command]
pub async fn move_file(from: String, to: String) -> Result<(), String> {
    fs_ops::move_file(&from, &to).await
}

#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let info = system::get_system_info();
    Ok(SystemInfo {
        os: info.os,
        arch: info.arch,
        hostname: info.hostname,
        memory_total: info.memory_total,
        memory_available: info.memory_available,
        cpu_count: info.cpu_count,
    })
}

#[command]
pub async fn show_notification(
    app: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn execute_command(
    command: String,
    args: Vec<String>,
) -> Result<ExecuteResult, String> {
    let config = ExecutorConfig::default();
    let result = executor::execute(
        &command,
        &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        None,
        &config,
    )
    .await?;

    Ok(ExecuteResult {
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
    })
}

#[command]
pub async fn get_clipboard_text(app: AppHandle) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_clipboard_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn check_for_updates(app: AppHandle) -> Result<Option<String>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    Ok(update.map(|u| u.version.to_string()))
}

#[command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    if let Some(update) = update {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
