// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod journal;
pub mod model;
pub mod reliability;

pub fn load_day_summaries() -> Vec<model::DaySummary> {
    let events = journal::collect_events();
    reliability::build_days(&events)
}

#[tauri::command]
fn load_days() -> Vec<model::DaySummary> {
    //load_day_summaries()
    return Vec::new()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_days])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
