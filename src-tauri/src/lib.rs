use audio::AudioEngine;
use log::{error, info};
use serde_json::json;
use tauri_plugin_store::StoreExt;
use tracing_subscriber::EnvFilter;
use std::ffi::c_void;
use std::{ffi::CStr, sync::Mutex};
use tauri::{LogicalSize, Manager, PhysicalSize, RunEvent};
use tauri_plugin_store::JsonValue;
use tracing_subscriber::fmt::time::LocalTime;
use vst3::base::funknown::IComponent_Impl;
use vst3::{base::funknown::IPlugView_Impl, gui::plug_view::PlatformType};
use vst3::{base::funknown::IPluginFactory_Impl, gui::plug_view::ViewRect};

use crate::plugins::PluginRegistry;

mod commands;
mod plugins;
mod settings;

type GlobalAudio = Mutex<AudioEngine>;
type GlobalPluginRegistry = Mutex<PluginRegistry>;

fn open_plugin(manager: &tauri::AppHandle, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let audio_state = manager.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();
    // engine.run();

    unsafe {
        let plugin_id = engine.load_plugin(path)?;
        let modules = engine.plugin_modules();

        // Get the first plugin (any plugin from the map)
        let plugin = modules.get(&plugin_id).unwrap();

        let window = tauri::WindowBuilder::new(manager, plugin_id).build()?;
        window.set_title(&plugin.name);
        let hwnd = window.hwnd()?.0;

        // plugin.component.unwrap().set_active(false);

        let view = plugin.view.unwrap();

        #[cfg(target_os = "windows")]
        view.attached(hwnd as *mut c_void, PlatformType::HWND);

        #[cfg(target_os = "macos")]
        view.attached(hwnd as *mut c_void, PlatformType::NSView);

        let mut rect = ViewRect::default();
        view.check_size_constraint(&mut rect);

        let new_size = PhysicalSize::new(rect.right, rect.bottom).to_logical::<i32>(window.scale_factor().unwrap());
        // LogicalSize::new(rect.right, rect.bottom).to_physical::<i32>(window.scale_factor().unwrap());
        let _ = window.set_size(new_size);

        window.clone().on_window_event(move |event| {
            if let tauri::WindowEvent::Resized(size) = event {
                let mut new_size = ViewRect::default();
                new_size.right = size.width as i32;
                new_size.bottom = size.height as i32;

                view.on_size(&mut new_size);

                view.check_size_constraint(&mut new_size);
                let new_size = PhysicalSize::new(new_size.right, new_size.bottom).to_logical::<i32>(window.scale_factor().unwrap());
                let _ = window.set_size(new_size);
            }
        });
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("trace"))
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
            commands::get_cpu_usage,
            commands::get_loaded_plugins
        ])
        .setup(|app| {
            app.manage(Mutex::new(settings::create_audio_engine_from_settings(app.app_handle())));

            let plugin_registry = settings::create_plugin_registry_from_settings(app.app_handle());

            for plugin in plugin_registry.get_discovered_plugins() {
                info!("Loading plugin: {:?}", plugin);
                let _ = open_plugin(app.app_handle(), plugin).map_err(|err| error!("{:?}", err));
            }
            app.manage(Mutex::new(plugin_registry));

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
