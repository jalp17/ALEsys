use tauri::{command, AppHandle};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_updater::UpdaterExt;
use serde::{Deserialize, Serialize};

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
    pub version: String,
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
    tokio::fs::read_to_string(&path)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn write_file(path: String, content: String) -> Result<(), String> {
    tokio::fs::write(&path, content.as_bytes())
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn list_directory(path: String) -> Result<Vec<FileItem>, String> {
    let mut entries = tokio::fs::read_dir(&path)
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        items.push(FileItem {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: Some(metadata.len()),
            modified: metadata.modified().ok().map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.to_rfc3339()
            }),
        });
    }

    Ok(items)
}

#[command]
pub async fn create_directory(path: String) -> Result<(), String> {
    tokio::fs::create_dir_all(&path)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn delete_file(path: String) -> Result<(), String> {
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|e| e.to_string())?;

    if metadata.is_dir() {
        tokio::fs::remove_dir_all(&path)
            .await
            .map_err(|e| e.to_string())
    } else {
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| e.to_string())
    }
}

#[command]
pub async fn copy_file(from: String, to: String) -> Result<(), String> {
    tokio::fs::copy(&from, &to)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub async fn move_file(from: String, to: String) -> Result<(), String> {
    tokio::fs::rename(&from, &to)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    let sys = sysinfo::System::new_all();

    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        version: std::env::consts::OS.to_string(),
        hostname: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        memory_total: sys.total_memory(),
        memory_available: sys.available_memory(),
        cpu_count: sys.cpus().len(),
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
    let output = std::process::Command::new(&command)
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(ExecuteResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
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
