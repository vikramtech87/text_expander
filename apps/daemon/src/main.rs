use config_manager::{watch_config_file, ConfigManager};
use engine::{Engine, SnippetSession};
use injector::Injector;
use parser::{Config};
use rdev::{grab, EventType, Key};
use std::sync::mpsc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::utils::{advance_session, AppEvent, LocalKeyEvent};

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
    let mut active_session: Option<SnippetSession> = None;

    // HARDENDED FIX: Create a thread-safe flag to tell our OS hook whether to swallow Tabs
    let session_active_flag = Arc::new(AtomicBool::new(false));

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

    // Keyboard listener thread
    let key_tx = event_tx.clone();
    let hook_flag = Arc::clone(&session_active_flag);

    thread::spawn(move || {
        println!("Global keyboard hook activated. Listening...");
        // if let Err(error) = listen(move |event| {
        //     if let Some(key_event) = parse_rdev_event(event) {
        //         let _ = key_tx.send(AppEvent::KeyEvent(key_event));
        //     }
        // }) {
        //     eprintln!("Failed to start keyboard hook: {:?}", error);
        // }

        let grab_result = grab(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                match key {
                    Key::Backspace => {
                        let _ = key_tx.send(AppEvent::KeyEvent(LocalKeyEvent::Backspace));
                        Some(event)
                    },
                    Key::Tab => {
                        if hook_flag.load(Ordering::SeqCst) {
                            let _ = key_tx.send(AppEvent::KeyEvent(LocalKeyEvent::Tab));
                            None // To prevent Tab getting printed in the text
                        } else {
                            Some(event)
                        }
                    }
                    _ => {
                        if let Some(actual_text) = event.name.clone() {
                            if !actual_text.is_empty() {
                                let _ = key_tx.send(AppEvent::KeyEvent(LocalKeyEvent::Text(actual_text)));
                            }
                        }
                        Some(event)
                    }
                }
            } else {
                Some(event)
            }
        });

        if let Err(error) = grab_result {
            eprintln!("Failed to start active keyboard grab: {:?}", error);
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
                            // If a session is active, we don't want to capture keys
                            if active_session.is_some() {
                                continue;
                            }

                            if let Some((snippets, trigger_len)) = engine.push_char(ch) {
                                // Trigger matched! Create a new session
                                let mut session = SnippetSession::new(snippets);

                                injector.delete_chars(trigger_len);

                                // Process the snippet parts up to the first placeholder
                                advance_session(&mut session, &mut injector);

                                if session.current_index < session.snippets.len() {
                                    active_session = Some(session);
                                    // Turn on tab swallowing!
                                    session_active_flag.store(true, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                    LocalKeyEvent::Backspace => {
                        if active_session.is_none() {
                            engine.handle_backspace();
                        }
                    }
                    LocalKeyEvent::Tab => {
                        if let Some(mut session) = active_session.take() {
                            advance_session(&mut session, &mut injector);
                            if session.current_index < session.snippets.len() {
                                active_session = Some(session);
                            } else {
                                // Turn off tab swallowing
                                session_active_flag.store(false, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
