use std::sync::Mutex;

use audio::AudioEngine;
use tauri::Manager;


type GlobalAudio = Mutex<AudioEngine>;

#[tauri::command]
pub fn get_hosts(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.cached_host_ids.iter().map(|id| id.name().to_string()).collect())
}

#[tauri::command]
pub fn get_input_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.cached_input_devices.get(&engine.host.id()).cloned().unwrap_or_default())
}

#[tauri::command]
pub fn get_output_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.cached_output_devices.get(&engine.host.id()).cloned().unwrap_or_default())
}