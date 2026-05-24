use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};
use sysinfo::System;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Default)]
struct WidgetConfigs(Mutex<HashMap<String, String>>);


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn toggle_always_on_top(window: tauri::Window) {
    let always_on_top: bool = window.is_always_on_top().unwrap_or(false);
    window
        .set_always_on_top(!always_on_top)
        .expect("Failed to set always on top");
}

#[tauri::command]
fn get_system_stats() -> serde_json::Value {
    thread_local! {
        static SYS: std::cell::RefCell<System> = std::cell::RefCell::new({
            let mut s = System::new_all();
            s.refresh_cpu_all();
            s.refresh_memory();
            s
        });
    }

    let (cpu_usage, ram_usage) = SYS.with(|sys_cell| {
        let mut sys = sys_cell.borrow_mut();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        
        let cpu = sys.global_cpu_usage();
        let total_mem = sys.total_memory() as f64;
        let used_mem = sys.used_memory() as f64;
        let ram = if total_mem > 0.0 {
            (used_mem / total_mem) * 100.0
        } else {
            0.0
        };
        (cpu, ram)
    });

    serde_json::json!({
        "cpu_usage": format!("{:.1}", cpu_usage),
        "ram_usage": format!("{:.1}", ram_usage)
    })
}

#[tauri::command]
fn save_widget_config(
    configs: tauri::State<'_, WidgetConfigs>,
    label: String,
    config: String,
) {
    // Only save the config. Window is created via JS API.
    if let Ok(mut map) = configs.0.lock() {
        map.insert(label.clone(), config.clone());
    }
}

#[tauri::command]
fn get_widget_config(configs: tauri::State<'_, WidgetConfigs>, label: String) -> Option<String> {
    if let Ok(map) = configs.0.lock() {
        map.get(&label).cloned()
    } else {
        None
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(WidgetConfigs::default())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            toggle_always_on_top,
            get_system_stats,
            save_widget_config,
            get_widget_config
        ])
        .setup(|app| {
            let show_menu = MenuItem::with_id(app, "show", "Open Dashboard", true, None::<&str>)?;
            let quit_menu = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_menu, &quit_menu])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .show_menu_on_left_click(true)
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
