use std::{error::Error, fmt, sync::Mutex};

use audio::AudioEngine;
use tauri::{ipc::InvokeError, Manager};


type GlobalAudio = Mutex<AudioEngine>;

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