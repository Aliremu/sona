use audio::AudioEngine;
use log::{error, info, trace};
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

use crate::commands::load_plugin;
use crate::plugins::PluginRegistry;

mod commands;
mod plugins;
mod settings;

type GlobalAudio = Mutex<AudioEngine>;
type GlobalPluginRegistry = Mutex<PluginRegistry>;

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
            commands::get_loaded_plugins,
            commands::load_plugin,
            commands::remove_plugin,
            commands::open_plugin_editor,
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
