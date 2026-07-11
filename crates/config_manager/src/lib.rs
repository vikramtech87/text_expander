pub mod errors;

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rule_codec::models::RulesConfig;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::{fs, thread};
use std::time::Duration;
use rule_codec::deserialize_rules;
use crate::errors::ConfigError;

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(path: PathBuf) -> Self {
        Self {
            config_path: path,
        }
    }
    
    pub fn path(&self) -> &Path {
        &self.config_path
    }

    pub fn load_config(&self) -> Result<RulesConfig, ConfigError> {
        let bytes = fs::read(&self.config_path)?;
        let rules = deserialize_rules(&bytes)?;

        Ok(RulesConfig { rules })
    }
}

pub fn watch_config_file(path: PathBuf, tx: Sender<RulesConfig>) -> Result<RecommendedWatcher, Box<dyn Error>> {
    let p = path.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
        let event = match res {
            Ok(event) => event,
            Err(err) => {
                eprintln!("Config file watch error: {:?}", err);
                return;
            }
        };

        if !matches!(event.kind, EventKind::Modify(_)) {
            return;
        }

        // Debounce step to allow OS write operations to finalize cleanly
        thread::sleep(Duration::from_millis(50));

        let read_bytes = match fs::read(&p) {
            Ok(read_bytes) => read_bytes,
            Err(err) => {
                eprintln!("[HOT RELOADING] Error reading config file: {:?}", err);
                return;
            }
        };
        let rules = match deserialize_rules(&read_bytes) {
            Ok(rules) => rules,
            Err(err) => {
                eprintln!("[HOT RELOADING] Error parsing bytes into rules {:?}", err);
                return;
            }
        };

        println!("🔄 Configuration changes detected! Hot-reloading rules...");
        let _ = tx.send(RulesConfig { rules });
    })?;

    watcher.watch(&path, RecursiveMode::NonRecursive)?;
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use rule_codec::{serialize_rules};
    use rule_codec::models::{ExpansionRule, ExpansionSnippet};
    use std::sync::mpsc::channel;

    #[test]
    fn test_load_config_reads_binary_correctly() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rtex_path = temp_dir.path().join("rules.rtex");

        let mock_rules = vec![ExpansionRule {
            trigger: ";test".to_string(),
            expansion: vec![
                ExpansionSnippet::Text {
                    content: "Pass".to_string(),
                }
            ]
        }];
        let binary_bytes = serialize_rules(&mock_rules);
        fs::write(&rtex_path, &binary_bytes).unwrap();

        let manager = ConfigManager {
            config_path: rtex_path,
        };
        let loaded_rules = manager.load_config().unwrap();

        assert_eq!(loaded_rules.rules.len(), 1);
        assert_eq!(loaded_rules.rules[0].trigger, ";test");
    }

    #[test]
    fn test_watch_config_file_hot_reloads_binary_changes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let rtex_path = temp_dir.path().join("rules.rtex");

        fs::write(&rtex_path, &[]).unwrap();

        let (tx, rx) = channel::<RulesConfig>();

        let _watcher = watch_config_file(rtex_path.clone(), tx).unwrap();

        let updated_rules = vec![ExpansionRule {
            trigger: ";hot".to_string(),
            expansion: vec![
                ExpansionSnippet::Text { content: "Reloaded".to_string() }
            ],
        }];
        fs::write(&rtex_path, serialize_rules(&updated_rules)).unwrap();

        let received_rules = rx.recv_timeout(Duration::from_millis(500))
            .expect("Failed to receive rules");

        assert_eq!(received_rules.rules.len(), 1);
        assert_eq!(received_rules.rules[0].trigger, ";hot");

    }
}