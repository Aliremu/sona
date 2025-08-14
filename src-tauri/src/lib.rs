use audio::AudioEngine;
use tauri::{Manager, PhysicalSize};
use tracing_subscriber::{fmt::time::LocalTime};
use vst3::base::funknown::IComponent_Impl;
use vst3::{base::funknown::IPlugView_Impl, gui::plug_view::PlatformType};
use vst3::{base::funknown::IPluginFactory_Impl, gui::plug_view::ViewRect};
use std::ffi::c_void;
use std::{ffi::CStr, sync::Mutex};

mod commands;

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
    
    Ok(format!("Created VST window with handle: {:p}", hwnd as *const c_void))
}

#[tauri::command]
async fn open_plugin_editor(app: tauri::AppHandle) -> Result<String, String> {
    match open_plugin(&app) {
        Ok(_) => Ok("Plugin editor opened successfully".to_string()),
        Err(e) => Err(format!("Failed to open plugin editor: {}", e)),
    }
}

type GlobalAudio = Mutex<AudioEngine>;

fn open_plugin(manager: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let audio_state = manager.state::<GlobalAudio>();
    let mut engine = audio_state.lock().unwrap();
    engine.run();
    
    let window = tauri::WindowBuilder::new(manager, "VST").build()?;
    let hwnd = window.hwnd()?.0;

    unsafe {
        engine.add_plugin("");
        let modules = engine.plugin_modules.read().unwrap();

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
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::get_input_devices, commands::get_output_devices, commands::get_hosts, create_vst_window, open_plugin_editor])
        .setup(|app| {
            app.manage(Mutex::new(AudioEngine::default()));
            let _ = open_plugin(&app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
