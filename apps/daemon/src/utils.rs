use rdev::{Event, EventType, Key};

pub enum LocalKeyEvent {
    Text(String),
    Tab,
    Backspace,
}

pub fn parse_rdev_event(event: Event) -> Option<LocalKeyEvent> {
    if let EventType::KeyPress(key) = event.event_type {
        match key {
            Key::Backspace => return Some(LocalKeyEvent::Backspace),
            Key::Tab => return Some(LocalKeyEvent::Tab),
            _ => {
                if let Some(actual_text) = event.name {
                    if !actual_text.is_empty() {
                        return Some(LocalKeyEvent::Text(actual_text));
                    }
                }
                None
            },
        }
    } else {
        None
    }
}

// pub fn handle_expansion(snippets: Vec<ExpansionSnippet>, injector: &mut Injector) {
//     let trigger_len = 4;
//     injector.delete_chars(trigger_len);
//
//     let mut output_text = String::new();
//     for snippet in snippets {
//         match snippet {
//             ExpansionSnippet::Text { content } => {
//                 output_text.push_str(&content);
//             }
//             ExpansionSnippet::Placeholder { .. }=> {
//                 output_text.push_str("[Interactive fields coming soon]");
//             }
//         }
//     }
//
//     if let Err(e) = injector.inject_text(&output_text) {
//         eprintln!("Failed to inject text: {:?}", e);
//     }
// }