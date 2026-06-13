use engine::SnippetSession;
use injector::Injector;
use parser::{Config, ExpansionSnippet};

pub enum AppEvent {
    KeyEvent(LocalKeyEvent),
    ConfigUpdate(Config),
}

pub enum LocalKeyEvent {
    Text(String),
    Tab,
    Backspace,
}

// pub fn parse_rdev_event(event: Event) -> Option<LocalKeyEvent> {
//     if let EventType::KeyPress(key) = event.event_type {
//         match key {
//             Key::Backspace => return Some(LocalKeyEvent::Backspace),
//             Key::Tab => return Some(LocalKeyEvent::Tab),
//             _ => {
//                 if let Some(actual_text) = event.name {
//                     if !actual_text.is_empty() {
//                         return Some(LocalKeyEvent::Text(actual_text));
//                     }
//                 }
//                 None
//             },
//         }
//     } else {
//         None
//     }
// }

pub fn advance_session(session: &mut SnippetSession, injector: &mut Injector) {
    while session.current_index < session.snippets.len() {
        let current_snippet = &session.snippets[session.current_index];

        match current_snippet {
            ExpansionSnippet::Text { content } => {
                let _ = injector.inject_text(content);
                session.current_index += 1;
            }
            ExpansionSnippet::Placeholder { .. } => {
                let display_text = current_snippet.get_default().unwrap();
                let select_len = display_text.chars().count();

                let _ = injector.inject_text(&display_text);
                injector.select_chars_backward(select_len);
                session.current_index += 1;
                break;
            }
        }
    }
}