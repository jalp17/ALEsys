use std::path::PathBuf;
use tauri::{
    command, AppHandle, Manager, Runtime, WebviewWindow,
};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_fs::{FsExt, read_dir};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_updater::UpdaterExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[command]
pub async fn open_file_dialog<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    options: Option<FileDialogOptions>,
) -> Result<Option<Vec<PathBuf>>, String> {
    let mut dialog = app.dialog().file();
    
    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(dir) = opts.default_path {
            dialog = dialog.set_directory(dir);
        }
        if let Some(filters) = opts.filters {
            for filter in filters {
                dialog = dialog.add_filter(filter.name, &filter.extensions);
            }
        }
        if opts.multiple.unwrap_or(false) {
            dialog = dialog.pick_files();
        } else {
            dialog = dialog.pick_file();
        }
    } else {
        dialog = dialog.pick_file();
    }

    dialog
        .blocking()
        .map_err(|e| e.to_string())
        .map(|paths| paths.map(|p| vec![p]))
}

#[derive(Deserialize, Debug)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub default_path: Option<PathBuf>,
    pub filters: Option<Vec<FileFilter>>,
    pub multiple: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[command]
pub async fn open_folder_dialog<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    options: Option<FileDialogOptions>,
) -> Result<Option<PathBuf>, String> {
    let mut dialog = app.dialog().file();
    
    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(dir) = opts.default_path {
            dialog = dialog.set_directory(dir);
        }
    }
    
    dialog
        .pick_folder()
        .blocking()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn save_file_dialog<R: Runtime>(
    app: AppHandle<R>,
    _window: WebviewWindow<R>,
    options: Option<FileDialogOptions>,
) -> Result<Option<PathBuf>, String> {
    let mut dialog = app.dialog().file();
    
    if let Some(opts) = options {
        if let Some(title) = opts.title {
            dialog = dialog.set_title(title);
        }
        if let Some(dir) = opts.default_path {
            dialog = dialog.set_directory(dir);
        }
        if let Some(filters) = opts.filters {
            for filter in filters {
                dialog = dialog.add_filter(filter.name, &filter.extensions);
            }
        }
    }
    
    dialog
        .save_file()
        .blocking()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn read_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<String, String> {
    let fs = app.fs();
    fs.read_to_string(&path).await.map_err(|e| e.to_string())
}

#[command]
pub async fn write_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    content: String,
) -> Result<(), String> {
    let fs = app.fs();
    fs.write(&path, content.as_bytes()).await.map_err(|e| e.to_string())
}

#[command]
pub async fn list_directory<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<Vec<FileItem>, String> {
    let fs = app.fs();
    let entries = fs.read_dir(&path).await.map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for entry in entries {
        let metadata = entry.metadata().await;
        let (size, modified) = if let Ok(m) = metadata {
            (Some(m.len()), m.modified().ok().map(|t| {
                chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()
            }))
        } else {
            (None, None)
        };
        
        items.push(FileItem {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            is_dir: entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false),
            size,
            modified,
        });
    }
    
    Ok(items)
}

#[command]
pub async fn create_directory<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<(), String> {
    let fs = app.fs();
    fs.create_dir_all(&path).await.map_err(|e| e.to_string())
}

#[command]
pub async fn delete_file<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<(), String> {
    let fs = app.fs();
    let metadata = fs.metadata(&path).await.map_err(|e| e.to_string())?;
    
    if metadata.is_dir() {
        fs.remove_dir_all(&path).await.map_err(|e| e.to_string())
    } else {
        fs.remove_file(&path).await.map_err(|e| e.to_string())
    }
}

#[command]
pub async fn copy_file<R: Runtime>(
    app: AppHandle<R>,
    from: String,
    to: String,
) -> Result<(), String> {
    let fs = app.fs();
    fs.copy(&from, &to).await.map_err(|e| e.to_string())
}

#[command]
pub async fn move_file<R: Runtime>(
    app: AppHandle<R>,
    from: String,
    to: String,
) -> Result<(), String> {
    let fs = app.fs();
    fs.rename(&from, &to).await.map_err(|e| e.to_string())
}

#[command]
pub async fn get_system_info<R: Runtime>(
    _app: AppHandle<R>,
) -> Result<SystemInfo, String> {
    let sys = sysinfo::System::new_all();
    
    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        version: std::env::consts::OS.to_string(),
        hostname: hostname::get().unwrap_or_default().to_string_lossy().to_string(),
        memory_total: sys.total_memory(),
        memory_available: sys.available_memory(),
        cpu_count: sys.cpus().len(),
    })
}

#[command]
pub async fn show_notification<R: Runtime>(
    app: AppHandle<R>,
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
pub async fn execute_command<R: Runtime>(
    _app: AppHandle<R>,
    command: String,
    args: Vec<String>,
) -> Result<ExecuteResult, String> {
    use std::process::Command;
    
    let output = Command::new(&command)
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
pub async fn get_clipboard_text<R: Runtime>(
    app: AppHandle<R>,
) -> Result<String, String> {
    app.clipboard()
        .read_text()
        .map_err(|e| e.to_string())
}

#[command]
pub async fn set_clipboard_text<R: Runtime>(
    app: AppHandle<R>,
    text: String,
) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| e.to_string())
}

#[command]
pub async fn check_for_updates<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<String>, String> {
    let updater = app.updater();
    let update = updater.check().await.map_err(|e| e.to_string())?;
    Ok(update.map(|u| u.version.to_string()))
}

#[command]
pub async fn install_update<R: Runtime>(
    app: AppHandle<R>,
) -> Result<(), String> {
    let updater = app.updater();
    updater.install().await.map_err(|e| e.to_string())
}
