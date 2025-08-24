#[cfg(target_os = "windows")]
use std::ffi::c_void;
use std::{error::Error, fmt, sync::Mutex};

use audio::{vst::host::PluginId, AudioEngine};
use log::trace;
use serde::{ser::SerializeStruct, Serialize};
use tauri::{ipc::InvokeError, Manager, PhysicalSize};
use vst3::gui::plug_view::ViewRect;
#[cfg(target_os = "windows")]
use vst3::{base::funknown::IPlugView_Impl, gui::plug_view::PlatformType};

use crate::plugins::PluginRegistry;

type GlobalAudio = Mutex<AudioEngine>;
type GlobalPluginRegistry = Mutex<PluginRegistry>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioError {
    HostError,
    InputDeviceError,
    OutputDeviceError,
    PluginLoadError,
    PluginEditorError,
}

impl Error for AudioError {}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::HostError => write!(f, "Host error"),
            AudioError::InputDeviceError => write!(f, "Input device error"),
            AudioError::OutputDeviceError => write!(f, "Output device error"),
            AudioError::PluginLoadError => write!(f, "Plugin load error"),
            AudioError::PluginEditorError => write!(f, "Plugin editor error"),
        }
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
    let engine = audio_state.lock().unwrap();

    Ok(engine.available_host_names())
}

#[tauri::command]
pub fn get_input_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    let Some(input_devices) = engine.cached_current_input_device_names() else {
        return Err(AudioError::InputDeviceError);
    };
    Ok(input_devices.to_vec())
}

#[tauri::command]
pub fn get_output_devices(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    let Some(output_devices) = engine.cached_current_output_device_names() else {
        return Err(AudioError::OutputDeviceError);
    };
    Ok(output_devices.to_vec())
}

/// Get current audio state
#[tauri::command]
pub fn get_host(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    Ok(engine.host_name().to_string())
}

#[tauri::command]
pub fn get_input_device(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    let Some(input_device) = engine.input_device_name() else {
        return Err(AudioError::InputDeviceError);
    };
    Ok(input_device)
}

#[tauri::command]
pub fn get_output_device(app_handle: tauri::AppHandle) -> Result<String, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    let Some(output_device) = engine.output_device_name() else {
        return Err(AudioError::OutputDeviceError);
    };
    Ok(output_device)
}

#[tauri::command]
pub fn get_buffer_size(app_handle: tauri::AppHandle) -> Result<u32, AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

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
    let registry = plugin_registry.lock().unwrap();

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
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    // Use the dialog with a callback instead of await
    app_handle
        .dialog()
        .file()
        .set_title("Select VST Plugin Directory")
        .pick_folder(move |result| match result {
            Some(path) => {
                let path_str = path
                    .as_path()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown path".to_string());
                let _ = app_handle.emit("directory-selected", path_str);
            }
            None => {
                let _ = app_handle.emit("directory-cancelled", ());
            }
        });

    Ok(())
}

#[tauri::command]
pub fn get_discovered_plugins(app_handle: tauri::AppHandle) -> Result<Vec<String>, AudioError> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let registry = plugin_registry.lock().unwrap();

    Ok(registry.get_discovered_plugins().to_vec())
}

#[tauri::command]
pub fn scan_plugins(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let plugin_registry = app_handle.state::<GlobalPluginRegistry>();
    let mut registry = plugin_registry.lock().unwrap();

    registry.scan_plugins()
}

#[tauri::command]
pub fn get_cpu_usage() -> Result<f32, String> {
    use sysinfo::System;

    let mut system = System::new_all();
    system.refresh_cpu(); // Refresh CPU information.

    // Wait a bit because CPU usage is calculated over time
    std::thread::sleep(std::time::Duration::from_millis(200));
    system.refresh_cpu();

    // Calculate average CPU usage across all cores
    let cpus = system.cpus();
    if cpus.is_empty() {
        return Err("No CPU information available".to_string());
    }

    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    let average_usage = total_usage / cpus.len() as f32;

    Ok(average_usage)
}

#[derive(Debug)]
pub struct PluginInfo {
    pub id: PluginId,
    pub name: String,
}

impl Serialize for PluginInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("PluginInfo", 2)?;
        state.serialize_field("id", &self.id.0)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

#[tauri::command]
pub fn get_loaded_plugins(app_handle: tauri::AppHandle) -> Result<Vec<PluginInfo>, AudioError> {
    trace!("Getting loaded plugins");
    let audio_state = app_handle.state::<GlobalAudio>();
    let engine = audio_state.lock().unwrap();

    let plugins = engine.plugin_modules();

    Ok(plugins
        .values()
        .map(|plugin| PluginInfo {
            id: plugin.id,
            name: plugin.name.clone(),
        })
        .collect())
}

#[tauri::command]
pub fn load_plugin(app_handle: tauri::AppHandle, path: &str) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .load_plugin(path)
        .map(|_| ())
        .map_err(|_| AudioError::PluginLoadError)
}

#[tauri::command]
pub fn remove_plugin(app_handle: tauri::AppHandle, plugin_id: u64) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();

    engine
        .remove_plugin(PluginId(plugin_id))
        .map(|_| ())
        .map_err(|_| AudioError::PluginLoadError)
}

#[tauri::command]
pub fn open_plugin_editor(app_handle: tauri::AppHandle, plugin_id: u64) -> Result<(), AudioError> {
    let audio_state = app_handle.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();
    let plugin_id = PluginId(plugin_id);

    unsafe {
        let mut modules = engine.plugin_modules_mut();

        // Get the first plugin (any plugin from the map)
        let plugin = modules.get_mut(&plugin_id).unwrap();

        let window = tauri::WindowBuilder::new(&app_handle, plugin_id)
            .build()
            .map_err(|_| AudioError::PluginEditorError)?;
        let _ = window.set_title(&plugin.name);
        let _ = window.set_resizable(false);

        #[cfg(target_os = "windows")]
        let hwnd = window.hwnd().map_err(|_| AudioError::PluginEditorError)?.0;
        #[cfg(target_os = "macos")]
        let hwnd = window
            .ns_view()
            .map_err(|_| AudioError::PluginEditorError)?
            .0;

        // plugin.component.unwrap().set_active(false);

        let view = plugin.view.unwrap();

        #[cfg(target_os = "windows")]
        view.attached(hwnd as *mut c_void, PlatformType::HWND);

        #[cfg(target_os = "macos")]
        view.attached(hwnd as *mut c_void, PlatformType::NSView);

        let mut rect = ViewRect::default();
        view.check_size_constraint(&mut rect);

        let scale_factor = window.scale_factor().unwrap();

        view.on_size(&mut rect);
        let new_size = PhysicalSize::new(rect.right, rect.bottom).to_logical::<i32>(scale_factor);
        let _ = window.set_size(new_size);

        let cloned_window = window.clone();

        plugin.set_window_resize_callback(move |view, new_size| {
            view.on_size(&mut *new_size);
            let _ = cloned_window
                .set_size(
                    PhysicalSize::new(new_size.right, new_size.bottom)
                        .to_logical::<i32>(scale_factor),
                )
                .unwrap();
        });

        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                view.removed();
            }
        });
    }

    Ok(())
}
