use audio::AudioEngine;
use log::info;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use crate::plugins::PluginRegistry;

pub fn create_audio_engine_from_settings(app: &tauri::AppHandle) -> AudioEngine {
    let store = app.store(".settings.json").unwrap();
    let mut engine = AudioEngine::default();

    let _ = store.get("audio-settings").and_then(|v| v.as_object().map(|obj| {
        obj.get("host").and_then(|v| v.as_str()).map(|s| engine.select_host(s).ok());
        obj.get("input").and_then(|v| v.as_str()).map(|s| engine.select_input(s).ok());
        obj.get("output").and_then(|v| v.as_str()).map(|s| engine.select_output(s).ok());
        obj.get("buffer_size").and_then(|v| v.as_u64()).map(|v| engine.set_buffer_size(v as u32).ok());
    }));

    engine
}

pub fn create_plugin_registry_from_settings(app: &tauri::AppHandle) -> PluginRegistry {
    let store = app.store(".settings.json").unwrap();
    let mut registry = PluginRegistry::new();

    let paths: Vec<String> = match store.get("plugin-paths") {
        Some(val) => {
            if let Some(arr) = val.as_array() {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                    .collect()
            } else {
                vec![
                    app.path().local_data_dir().unwrap().join("Programs/Common/VST3").to_string_lossy().into_owned(),
                    "/Program Files/Common Files/VST3".to_string(),
                    "/Program Files (x86)/Common Files/VST3".to_string(),
                    app.path().app_local_data_dir().unwrap().join("/VST3").to_string_lossy().into_owned(),
                ]
            }
        },
        None => vec![
                    app.path().local_data_dir().unwrap().join("Programs/Common/VST3").to_string_lossy().into_owned(),
                    "/Program Files/Common Files/VST3".to_string(),
                    "/Program Files (x86)/Common Files/VST3".to_string(),
                    app.path().app_local_data_dir().unwrap().join("/VST3").to_string_lossy().into_owned(),
                    "C:\\Coding\\Projects\\lyre\\plugins".to_string()
                ]
    };

    registry.set_plugin_paths(paths);
    let _ = registry.scan_plugins();

    registry
}