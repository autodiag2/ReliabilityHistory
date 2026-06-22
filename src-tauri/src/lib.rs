pub mod package;
use crate::package::package::{PackageDB, PackageInfo};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod journal;
pub mod model;
pub mod reliability;

#[tauri::command]
async fn load_days() -> Vec<model::DaySummary> {
    let events = journal::collect_events();
    return reliability::build_days(&events)
}

#[tauri::command]
async fn exec_retrieve_packages_info(path: String) -> Vec<PackageInfo> {
    let db = PackageDB::new();
    return match db.retrieve(path.as_str()) {
        None => Vec::new(),
        Some(packages) => packages
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_days, exec_retrieve_packages_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
