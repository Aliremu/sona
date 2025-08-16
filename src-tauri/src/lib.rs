use audio::AudioEngine;
use log::info;
use serde_json::json;
use tauri_plugin_store::StoreExt;
use std::ffi::c_void;
use std::{ffi::CStr, sync::Mutex};
use tauri::{Manager, PhysicalSize, RunEvent};
use tauri_plugin_store::JsonValue;
use tracing_subscriber::fmt::time::LocalTime;
use vst3::base::funknown::IComponent_Impl;
use vst3::{base::funknown::IPlugView_Impl, gui::plug_view::PlatformType};
use vst3::{base::funknown::IPluginFactory_Impl, gui::plug_view::ViewRect};

use crate::plugins::PluginRegistry;

mod commands;
mod plugins;
mod settings;

#[tauri::command]
async fn create_vst_window(app: tauri::AppHandle) -> Result<String, String> {
    let window = tauri::WindowBuilder::new(&app, "vst_window")
        .title("VST Plugin")
        .inner_size(800.0, 600.0)
        .resizable(true)
        .decorations(true)
        .build()
        .map_err(|e| e.to_string())?;

    // Get the native window handle for embedding VST UIs
    #[cfg(target_os = "windows")]
    let hwnd = window.hwnd().map_err(|e| e.to_string())?.0;

    #[cfg(not(target_os = "windows"))]
    let hwnd = 0;

    Ok(format!(
        "Created VST window with handle: {:p}",
        hwnd as *const c_void
    ))
}

#[tauri::command]
async fn open_plugin_editor(app: tauri::AppHandle) -> Result<String, String> {
    match open_plugin(&app) {
        Ok(_) => Ok("Plugin editor opened successfully".to_string()),
        Err(e) => Err(format!("Failed to open plugin editor: {}", e)),
    }
}

type GlobalAudio = Mutex<AudioEngine>;
type GlobalPluginRegistry = Mutex<PluginRegistry>;

fn open_plugin(manager: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let audio_state = manager.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();
    // engine.run();

    let window = tauri::WindowBuilder::new(manager, "VST").build()?;
    let hwnd = window.hwnd()?.0;

    unsafe {
        engine.add_plugin("C:\\Coding\\Projects\\lyre\\plugins\\Archetype Nolly.vst3");
        let modules = engine.plugin_modules();

        let plugin = modules.first().expect("Could not get plugin 0!");

        plugin.component.unwrap().set_active(false);

        let view = plugin.view.unwrap();

        let name = plugin.factory.unwrap().get_class_info(0).unwrap().name;

        let _ = window.set_title(CStr::from_ptr(name.as_ptr()).to_str().unwrap());

        #[cfg(target_os = "windows")]
        view.attached(hwnd as *mut c_void, PlatformType::HWND);

        #[cfg(target_os = "macos")]
        view.attached(hwnd as *mut c_void, PlatformType::NSView);

        let mut rect = ViewRect::default();
        view.check_size_constraint(&mut rect);
        let _ = window.set_size(PhysicalSize::new(rect.right, rect.bottom));

        window.clone().on_window_event(move |event| {
            if let tauri::WindowEvent::Resized(size) = event {
                let mut rect = ViewRect::default();
                view.check_size_constraint(&mut rect);

                let _ = window.set_size(PhysicalSize::new(rect.right, rect.bottom));
            }
        });
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        // .with_env_filter(EnvFilter::new("sona=info"))
        .with_timer(LocalTime::rfc_3339())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_hosts,
            commands::get_input_devices,
            commands::get_output_devices,
            commands::get_host,
            commands::get_input_device,
            commands::get_output_device,
            commands::get_buffer_size,
            commands::select_host,
            commands::select_input,
            commands::select_output,
            commands::set_buffer_size,
            commands::get_plugin_paths,
            commands::set_plugin_paths,
            commands::get_discovered_plugins,
            commands::browse_directory,
            commands::scan_plugins,
            create_vst_window,
            open_plugin_editor
        ])
        .setup(|app| {
            app.manage(Mutex::new(settings::create_audio_engine_from_settings(app.app_handle())));
            app.manage(Mutex::new(settings::create_plugin_registry_from_settings(app.app_handle())));

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(move |app, event| {
            match &event {
                RunEvent::ExitRequested { code, api, .. } => {
                    info!("Goodbye...");
                    let audio_state = app.state::<GlobalAudio>();
                    let engine = audio_state.lock().unwrap();
                    let plugin_registry = app.state::<GlobalPluginRegistry>();

                    let store = app.store(".settings.json").unwrap();
                    store.set("audio-settings", json!({
                        "host": engine.host_name(),
                        "input": engine.input_device_name(),
                        "output": engine.output_device_name(),
                        "buffer_size": engine.buffer_size()
                    }));

                    store.set("plugin-paths", plugin_registry.lock().unwrap().get_plugin_paths());

                    store.save().unwrap();
                    store.close_resource();
                },

                _ => {}
            };
        });
}
