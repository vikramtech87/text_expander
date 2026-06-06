use rdev::{Event, EventType, Key};
use parser::Config;

pub enum AppEvent {
    KeyEvent(LocalKeyEvent),
    ConfigUpdate(Config),
}

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