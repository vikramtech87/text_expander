use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use parser::Config;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::{fs, thread};
use std::time::Duration;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let base_dir = dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("."));

        let config_dir = base_dir.join("rust_text_expander");
        let config_path = config_dir.join("rules.toml");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let manager = Self { config_path };
        manager.ensure_default_config_exists()?;

        Ok(manager)
    }

    pub fn path(&self) -> &Path {
        &self.config_path
    }

    pub fn load_config(&self) -> Result<Config, Box<dyn Error>> {
        let content = std::fs::read_to_string(&self.config_path)?;
        let config = Config::parse(&content)?;
        Ok(config)
    }

    fn ensure_default_config_exists(&self) -> Result<(), std::io::Error> {
        if !self.config_path.exists() {
            let default_toml = r#"
[[rules]]
trigger = ";gme"
expansion = [
    { type = "text", content = "Gemini is awesome!" }
]

[[rules]]
trigger = ";shrug"
expansion = [
    { type = "text", content = '¯\_(ツ)_/¯' }
]
            "#;

            std::fs::write(&self.config_path, default_toml)?;
            println!("Created default configuration file at {:?}", self.config_path);
        }
        Ok(())
    }
}

pub fn watch_config_file(path: PathBuf, tx: Sender<Config>) -> Result<RecommendedWatcher, Box<dyn Error>> {
    let p = path.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        let event = match res {
            Ok(event) => event,
            Err(e) => {
                eprintln!("File watcher error: {:?}", e);
                return;
            }
        };

        if !matches!(event.kind, EventKind::Modify(_)) {
            return;
        }

        // 🟢 Debounce step to allow OS write operations to finalize cleanly
        thread::sleep(Duration::from_millis(50));

        let processing_pipeline = fs::read_to_string(&p)
            .map_err(|_| "Failed to read configruation file path")
            .and_then(|content| {
                // 🟢 Uses your unified parser serialization to decode the rules
                toml::from_str::<Config>(&content)
                    .map_err(|_| "Failed to parse TOML formatting")
            });

        match processing_pipeline {
            Ok(new_config) => {
                println!("🔄 Configuration changes detected! Hot-reloading rules...");
                let _ = tx.send(new_config);
            }
            Err(e) => eprintln!("⚠️ Reload abort: {}", e)
        }
    })?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}