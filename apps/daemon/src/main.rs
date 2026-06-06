use std::sync::mpsc;
use std::thread;
use rdev::listen;
use engine::Engine;
use injector::Injector;
use parser::{Config, ExpansionSnippet};

use crate::utils::{parse_rdev_event, LocalKeyEvent};

mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Starting TextExpander Daemon");

    let mock_toml = r#"
        [[rules]]
        trigger = ";gme"
        expansion = [
            { type = "text", content = "Gemini is an awesome AI teammate!" }
        ]
    "#;

    let config = Config::parse(mock_toml)?;
    let mut engine = Engine::new(config);
    let mut injector = Injector::new()?;

    let (tx, rx) = mpsc::channel::<LocalKeyEvent>();

    thread::spawn(move || {
        println!("Global keyboard hook activated. Listening...");
        if let Err(error) = listen(move |event| {
            if let Some(key_event) = parse_rdev_event(event) {
                let _ = tx.send(key_event);
            }
        }) {
            eprintln!("Failed to start keyboard hook: {:?}", error);
        }
    });

    for event in rx {
        match event {
            LocalKeyEvent::Text(text) => {
                for ch in text.chars() {
                    if let Some(snippets) = engine.push_char(ch) {
                        // Todo! delete the exact number of characters
                        injector.delete_chars(4);

                        let mut output_string = String::new();
                        for snippet in snippets {
                            match snippet {
                                ExpansionSnippet::Text { content } => output_string.push_str(&content),
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

    Ok(())
}