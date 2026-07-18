mod commands;

use tauri::{
    Emitter,
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    run();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::open_file_dialog,
            commands::open_folder_dialog,
            commands::save_file_dialog,
            commands::read_file,
            commands::write_file,
            commands::list_directory,
            commands::create_directory,
            commands::delete_file,
            commands::copy_file,
            commands::move_file,
            commands::get_system_info,
            commands::show_notification,
            commands::execute_command,
            commands::get_clipboard_text,
            commands::set_clipboard_text,
            commands::check_for_updates,
            commands::install_update,
        ])
        .setup(|app| {
            // Create main menu
            let app_menu = SubmenuBuilder::new(app, "ALEsys")
                .item(&MenuItemBuilder::with_id("about", "About ALEsys").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("preferences", "Preferences...").accelerator("CmdOrCtrl+,").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("check_updates", "Check for Updates...").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("quit", "Quit").accelerator("CmdOrCtrl+Q").build(app)?)
                .build()?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&MenuItemBuilder::with_id("new_file", "New File").accelerator("CmdOrCtrl+N").build(app)?)
                .item(&MenuItemBuilder::with_id("open_file", "Open File...").accelerator("CmdOrCtrl+O").build(app)?)
                .item(&MenuItemBuilder::with_id("open_folder", "Open Folder...").accelerator("CmdOrCtrl+Shift+O").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("save", "Save").accelerator("CmdOrCtrl+S").build(app)?)
                .item(&MenuItemBuilder::with_id("save_as", "Save As...").accelerator("CmdOrCtrl+Shift+S").build(app)?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&MenuItemBuilder::with_id("undo", "Undo").accelerator("CmdOrCtrl+Z").build(app)?)
                .item(&MenuItemBuilder::with_id("redo", "Redo").accelerator("CmdOrCtrl+Shift+Z").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("cut", "Cut").accelerator("CmdOrCtrl+X").build(app)?)
                .item(&MenuItemBuilder::with_id("copy", "Copy").accelerator("CmdOrCtrl+C").build(app)?)
                .item(&MenuItemBuilder::with_id("paste", "Paste").accelerator("CmdOrCtrl+V").build(app)?)
                .item(&MenuItemBuilder::with_id("select_all", "Select All").accelerator("CmdOrCtrl+A").build(app)?)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&MenuItemBuilder::with_id("toggle_sidebar", "Toggle Sidebar").accelerator("CmdOrCtrl+B").build(app)?)
                .item(&MenuItemBuilder::with_id("toggle_terminal", "Toggle Terminal").accelerator("CmdOrCtrl+`").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("zoom_in", "Zoom In").accelerator("CmdOrCtrl+=").build(app)?)
                .item(&MenuItemBuilder::with_id("zoom_out", "Zoom Out").accelerator("CmdOrCtrl+-").build(app)?)
                .item(&MenuItemBuilder::with_id("zoom_reset", "Reset Zoom").accelerator("CmdOrCtrl+0").build(app)?)
                .build()?;

            let run_menu = SubmenuBuilder::new(app, "Run")
                .item(&MenuItemBuilder::with_id("run_code", "Run Code").accelerator("CmdOrCtrl+Enter").build(app)?)
                .item(&MenuItemBuilder::with_id("stop_code", "Stop").accelerator("CmdOrCtrl+.").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("run_terminal", "Run in Terminal").accelerator("CmdOrCtrl+Shift+Enter").build(app)?)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&MenuItemBuilder::with_id("docs", "Documentation").build(app)?)
                .item(&MenuItemBuilder::with_id("shortcuts", "Keyboard Shortcuts").accelerator("CmdOrCtrl+K").build(app)?)
                .item(&MenuItemBuilder::with_id("report_issue", "Report Issue").build(app)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &run_menu, &help_menu])
                .build()?;
            app.set_menu(menu)?;

            // Handle menu events
            app.on_menu_event(move |app, event| {
                let id = event.id().as_ref();
                if id == "quit" {
                    app.exit(0);
                    return;
                }
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("menu-action", id);
                }
            });

            // Create system tray
            let quit_item = MenuItemBuilder::with_id("tray_quit", "Quit ALEsys").build(app)?;
            let show_item = MenuItemBuilder::with_id("tray_show", "Show").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "tray_quit" => app.exit(0),
                    "tray_show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Register global shortcuts
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;

                let _ = app.global_shortcut().on_shortcut("Super+K", move |_app, _shortcut, _event| {
                    if let Some(window) = _app.get_webview_window("main") {
                        let _ = window.emit("global-shortcut", "command_palette");
                    }
                });
                let _ = app.global_shortcut().on_shortcut("Super+P", move |_app, _shortcut, _event| {
                    if let Some(window) = _app.get_webview_window("main") {
                        let _ = window.emit("global-shortcut", "quick_open");
                    }
                });
                let _ = app.global_shortcut().on_shortcut("Super+Shift+P", move |_app, _shortcut, _event| {
                    if let Some(window) = _app.get_webview_window("main") {
                        let _ = window.emit("global-shortcut", "quick_open_files");
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    #[cfg(desktop)]
                    {
                        if window.app_handle().tray_by_id("main").is_some() {
                            let _ = window.hide();
                            api.prevent_close();
                        }
                    }
                }
                tauri::WindowEvent::Focused(focused) => {
                    let _ = window.emit("window-focused", focused);
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
