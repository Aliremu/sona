use log::info;

pub struct PluginRegistry {
    plugin_paths: Vec<String>,
    plugins: Vec<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { 
            plugin_paths: Vec::new(),
            plugins: Vec::new() 
        }
    }

    // Helper function to clean up Windows UNC paths
    fn clean_path(canonical_path: std::path::PathBuf) -> String {
        let path_str = canonical_path.to_string_lossy().to_string();
        
        // Remove Windows UNC prefix if present
        if path_str.starts_with(r"\\?\") {
            path_str.strip_prefix(r"\\?\").unwrap_or(&path_str).to_string()
        } else {
            path_str
        }
    }

    pub fn add_plugin_path(&mut self, path: String) -> Result<(), String> {
        let path_buf = std::path::Path::new(&path);

        if !path_buf.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        if !path_buf.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        let canonical_path = path_buf.canonicalize()
            .map_err(|e| format!("Failed to canonicalize path '{}': {}", path, e))?;
        
        let clean_path = Self::clean_path(canonical_path);
        self.plugin_paths.push(clean_path);
        Ok(())
    }

    pub fn remove_plugin_path(&mut self, path: &str) {
        self.plugin_paths.retain(|p| p != path);
    }

    pub fn set_plugin_paths(&mut self, paths: Vec<String>) -> Result<(), String> {
        self.plugin_paths.clear();
        
        for path in paths {
            self.add_plugin_path(path)?;
        }
        
        Ok(())
    }

    pub fn get_plugin_paths(&self) -> &[String] {
        &self.plugin_paths
    }

    pub fn scan_plugins(&mut self) -> Result<Vec<String>, String> {
        // Clear existing plugins before scanning
        self.plugins.clear();
        
        for path in self.plugin_paths.clone() {
            let entries = std::fs::read_dir(path).map_err(|e| e.to_string());
            let Ok(entries) = entries else {
              info!("Failed to read dir: {}", entries.unwrap_err());
              continue;
            };

            for entry in entries {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().extension().map_or(false, |ext| ext == "vst3") {
                    self.add_plugin(entry.path().to_string_lossy().into_owned());
                }
            }
        }
        
        Ok(self.get_discovered_plugins().to_vec())
    }

    pub fn add_plugin(&mut self, plugin: String) {
        self.plugins.push(plugin);
    }

    pub fn remove_plugin(&mut self, plugin: &str) {
        self.plugins.retain(|p| p != plugin);
    }

    pub fn get_discovered_plugins(&self) -> &[String] {
        &self.plugins
    }
}