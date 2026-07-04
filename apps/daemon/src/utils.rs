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