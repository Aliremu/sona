use std::{error::Error, fmt, sync::Mutex};

use audio::AudioEngine;
use tauri::{ipc::InvokeError, Manager};

use crate::plugins::PluginRegistry;

type GlobalAudio = Mutex<AudioEngine>;
type GlobalPluginRegistry = Mutex<PluginRegistry>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioError {
    HostError,
    InputDeviceError,
    OutputDeviceError,
}

impl Error for AudioError {}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<AudioError> for InvokeError {
    fn from(error: AudioError) -> Self {
        InvokeError::from(error.to_string())
    }
}

#[tauri::command]
pub fn get_hosts(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.available_host_names())
}

#[tauri::command]
pub fn get_input_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    let Some(input_devices) = engine.cached_current_input_device_names() else {
        return Err(AudioError::InputDeviceError);
    };
    Ok(input_devices.to_vec())
}

#[tauri::command]
pub fn get_output_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    let Some(output_devices) = engine.cached_current_output_device_names() else {
        return Err(AudioError::OutputDeviceError);
    };
    Ok(output_devices.to_vec())
}

/// Get current audio state
#[tauri::command]
pub fn get_host(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.host_name().to_string())
}

#[tauri::command]
pub fn get_input_device(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    let Some(input_device) = engine.input_device_name() else {
        return Err(AudioError::InputDeviceError);
    };
    Ok(input_device)
}

#[tauri::command]
pub fn get_output_device(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    let Some(output_device) = engine.output_device_name() else {
        return Err(AudioError::OutputDeviceError);
    };
    Ok(output_device)
}

#[tauri::command]
pub fn get_buffer_size(app_handle: tauri::AppHandle) -> Result<u32, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    Ok(engine.buffer_size())
}

/// Set current audio states
#[tauri::command]
pub fn select_host(app_handle: tauri::AppHandle, host: String) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .select_host(&host)
        .and_then(|_| engine.run())
        .map_err(|_| AudioError::HostError)
}

#[tauri::command]
pub fn select_input(app_handle: tauri::AppHandle, input_device: String) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .select_input(&input_device)
        .and_then(|_| engine.run())
        .map_err(|_| AudioError::InputDeviceError)
}

#[tauri::command]
pub fn select_output(
    app_handle: tauri::AppHandle,
    output_device: String,
) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .select_output(&output_device)
        .and_then(|_| engine.run())
        .map_err(|_| AudioError::OutputDeviceError)
}

#[tauri::command]
pub fn set_buffer_size(app_handle: tauri::AppHandle, size: u32) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .set_buffer_size(size)
        .and_then(|_| engine.run())
        .map_err(|_| AudioError::HostError)
}

#[tauri::command]
pub fn get_plugin_paths(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let mut registry = plugin_registry.lock().unwrap();

    Ok(registry.get_plugin_paths().to_vec())
}

#[tauri::command]
pub fn set_plugin_paths(app_handle: tauri::AppHandle, paths: Vec<String>) -> Result<(), String> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let mut registry = plugin_registry.lock().unwrap();

    registry.set_plugin_paths(paths)?;
    Ok(())
}

#[tauri::command]
pub fn browse_directory(app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    use tauri::Emitter;
    
    // Use the dialog with a callback instead of await
    app_handle.dialog()
        .file()
        .set_title("Select VST Plugin Directory")
        .pick_folder(move |result| {
            match result {
                Some(path) => {
                    // Emit an event with the selected path
                    let path_str = path.as_path().map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Unknown path".to_string());
                    let _ = app_handle.emit("directory-selected", path_str);
                },
                None => {
                    // User cancelled, emit a cancelled event
                    let _ = app_handle.emit("directory-cancelled", ());
                },
            }
        });
    
    Ok(())
}

#[tauri::command]
pub fn get_discovered_plugins(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let mut registry = plugin_registry.lock().unwrap();

    Ok(registry.get_discovered_plugins().to_vec())
}

#[tauri::command]
pub fn scan_plugins(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let mut registry = plugin_registry.lock().unwrap();

    registry.scan_plugins()
}
