//! ALEsys Desktop - GraphRAG-PG IDE
//!
//! Native desktop application built with Tauri 2.0

mod commands;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, WebviewUrl, WebviewWindowBuilder,
};

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
                .item(&MenuItemBuilder::new("About ALEsys").id("about").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Preferences...").id("preferences").accelerator("CmdOrCtrl+,").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Check for Updates...").id("check_updates").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Quit").id("quit").accelerator("CmdOrCtrl+Q").build(app)?)
                .build()?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&MenuItemBuilder::new("New File").id("new_file").accelerator("CmdOrCtrl+N").build(app)?)
                .item(&MenuItemBuilder::new("Open File...").id("open_file").accelerator("CmdOrCtrl+O").build(app)?)
                .item(&MenuItemBuilder::new("Open Folder...").id("open_folder").accelerator("CmdOrCtrl+Shift+O").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Save").id("save").accelerator("CmdOrCtrl+S").build(app)?)
                .item(&MenuItemBuilder::new("Save As...").id("save_as").accelerator("CmdOrCtrl+Shift+S").build(app)?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&MenuItemBuilder::new("Undo").id("undo").accelerator("CmdOrCtrl+Z").build(app)?)
                .item(&MenuItemBuilder::new("Redo").id("redo").accelerator("CmdOrCtrl+Shift+Z").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Cut").id("cut").accelerator("CmdOrCtrl+X").build(app)?)
                .item(&MenuItemBuilder::new("Copy").id("copy").accelerator("CmdOrCtrl+C").build(app)?)
                .item(&MenuItemBuilder::new("Paste").id("paste").accelerator("CmdOrCtrl+V").build(app)?)
                .item(&MenuItemBuilder::new("Select All").id("select_all").accelerator("CmdOrCtrl+A").build(app)?)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&MenuItemBuilder::new("Toggle Sidebar").id("toggle_sidebar").accelerator("CmdOrCtrl+B").build(app)?)
                .item(&MenuItemBuilder::new("Toggle Terminal").id("toggle_terminal").accelerator("CmdOrCtrl+`").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Zoom In").id("zoom_in").accelerator("CmdOrCtrl+=").build(app)?)
                .item(&MenuItemBuilder::new("Zoom Out").id("zoom_out").accelerator("CmdOrCtrl+-").build(app)?)
                .item(&MenuItemBuilder::new("Reset Zoom").id("zoom_reset").accelerator("CmdOrCtrl+0").build(app)?)
                .build()?;

            let run_menu = SubmenuBuilder::new(app, "Run")
                .item(&MenuItemBuilder::new("Run Code").id("run_code").accelerator("CmdOrCtrl+Enter").build(app)?)
                .item(&MenuItemBuilder::new("Stop").id("stop_code").accelerator("CmdOrCtrl+.").build(app)?)
                .separator()
                .item(&MenuItemBuilder::new("Run in Terminal").id("run_terminal").accelerator("CmdOrCtrl+Shift+Enter").build(app)?)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&MenuItemBuilder::new("Documentation").id("docs").build(app)?)
                .item(&MenuItemBuilder::new("Keyboard Shortcuts").id("shortcuts").accelerator("CmdOrCtrl+K").build(app)?)
                .item(&MenuItemBuilder::new("Report Issue").id("report_issue").build(app)?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &run_menu, &help_menu])
                .build()?;
            app.set_menu(menu)?;

            // Handle menu events
            app.on_menu_event(|app, event| {
                match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "about" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "about");
                        }
                    }
                    "preferences" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "preferences");
                        }
                    }
                    "check_updates" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "check_updates");
                        }
                    }
                    "new_file" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "new_file");
                        }
                    }
                    "open_file" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "open_file");
                        }
                    }
                    "open_folder" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "open_folder");
                        }
                    }
                    "save" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "save");
                        }
                    }
                    "save_as" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "save_as");
                        }
                    }
                    "toggle_sidebar" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "toggle_sidebar");
                        }
                    }
                    "toggle_terminal" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "toggle_terminal");
                        }
                    }
                    "run_code" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "run_code");
                        }
                    }
                    "shortcuts" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("menu-action", "shortcuts");
                        }
                    }
                    _ => {}
                }
            });

            // Create system tray
            let quit_item = MenuItemBuilder::new("Quit ALEsys")
                .id("tray_quit")
                .build(app)?;
            let show_item = MenuItemBuilder::new("Show")
                .id("tray_show")
                .build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .items(&[&show_item, &quit_item])
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
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
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Code, Modifiers};
                
                let app_handle = app.handle().clone();
                let _ = app.global_shortcut().on_shortcut(move |app, shortcut, _event| {
                    if shortcut.matches(Modifiers::CONTROL, Code::KeyK) {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("global-shortcut", "command_palette");
                        }
                    } else if shortcut.matches(Modifiers::CONTROL, Code::KeyP) {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("global-shortcut", "quick_open");
                        }
                    } else if shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyP) {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("global-shortcut", "quick_open_files");
                        }
                    }
                });
                
                let _ = app.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL), Code::KeyK));
                let _ = app.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL), Code::KeyP));
                let _ = app.global_shortcut().register(Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyP));
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Hide instead of close if tray is available
                    #[cfg(desktop)]
                    {
                        if window.app_handle().tray_by_id("main").is_some() {
                            let _ = window.hide();
                            api.prevent_close();
                        }
                    }
                }
                tauri::WindowEvent::Focused(focused) => {
                    if focused {
                        let _ = window.emit("window-focused", true);
                    } else {
                        let _ = window.emit("window-focused", false);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
