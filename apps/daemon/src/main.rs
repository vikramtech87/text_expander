use config_manager::{watch_config_file, ConfigManager};
use engine::Engine;
use injector::Injector;
use parser::{Config, ExpansionSnippet};
use rdev::listen;
use std::sync::mpsc;
use std::thread;

use crate::utils::{parse_rdev_event, AppEvent, LocalKeyEvent};

mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Starting TextExpander Daemon");

    // 1. Initialize our cross-thread channel
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();

    // 2. Setup Config Manager and Load Initial Rules.
    let config_mgr = ConfigManager::new()?;
    let initial_config = config_mgr.load_config()?;
    let mut engine = Engine::new(initial_config);
    let mut injector = Injector::new()?;

    // 3. Spawn File Watcher
    let (cfg_tx, cfg_rx) = mpsc::channel::<Config>();
    let _watcher = watch_config_file(config_mgr.path().to_path_buf(), cfg_tx)?;

    // Let's optimize the watcher linkage. To keep it clean, we can pass our main event_tx
    // directly to a revised watch_config_file hook, or handle it via a proxy thread.
    // Let's spin up a dedicated proxy channel for config updates to keep libraries clean:
    let loop_tx = event_tx.clone();
    thread::spawn(move || {
        for new_config in cfg_rx {
            let _ = loop_tx.send(AppEvent::ConfigUpdate(new_config));
        }
    });

    let key_tx = event_tx.clone();
    thread::spawn(move || {
        println!("Global keyboard hook activated. Listening...");
        if let Err(error) = listen(move |event| {
            if let Some(key_event) = parse_rdev_event(event) {
                let _ = key_tx.send(AppEvent::KeyEvent(key_event));
            }
        }) {
            eprintln!("Failed to start keyboard hook: {:?}", error);
        }
    });

    for event in event_rx {
        match event {
            AppEvent::ConfigUpdate(new_config) => {
                engine.update_config(new_config);
            }
            AppEvent::KeyEvent(key) => {
                match key {
                    LocalKeyEvent::Text(text) => {
                        for ch in text.chars() {
                            if let Some(snippets) = engine.push_char(ch) {
                                // Todo! delete the exact number of characters
                                injector.delete_chars(7);

                                let mut output_string = String::new();
                                for snippet in snippets {
                                    match snippet {
                                        ExpansionSnippet::Text { content } => {
                                            output_string.push_str(&content)
                                        }
                                        ExpansionSnippet::Placeholder { name, .. } => {
                                            output_string.push_str(&format!("[{}]", name));
                                        }
                                    }
                                }
                                let _ = injector.inject_text(&output_string);
                            }
                        }
                    }
                    LocalKeyEvent::Backspace => {
                        engine.handle_backspace();
                    }
                    LocalKeyEvent::Tab => {
                        println!("Tab detected! (We can use this later to jump placeholders)");
                        // Right now, do nothing so the OS handles it normally
                    }
                }
            }
        }
    }

    Ok(())
}
