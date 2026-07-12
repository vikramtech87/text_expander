use tray_icon::{Icon, TrayIconEvent, menu::{MenuEvent, MenuId}, TrayIcon};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use engine::SnippetSession;
use injector::Injector;
use rule_codec::models::{RulesConfig, ExpansionSnippet};

pub enum AppEvent {
    KeyEvent(LocalKeyEvent),
    ConfigUpdate(RulesConfig),
}

pub enum LocalKeyEvent {
    Text(String),
    Tab,
    Backspace,
    Escape,
}

#[derive(Debug)]
pub enum SystemTrayMessage {
    Menu(MenuEvent),
    Tray(TrayIconEvent)
}

#[allow(dead_code)]
pub struct DaemonApp {
    pub tray_icon: Option<TrayIcon>,
    pub quit_item_id: MenuId,
}

impl ApplicationHandler<SystemTrayMessage> for DaemonApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: SystemTrayMessage) {
        match event {
            SystemTrayMessage::Menu(event) => {
                if event.id == self.quit_item_id {
                    println!("Stopping textexpander...");
                    event_loop.exit();
                }
            }
            SystemTrayMessage::Tray(_event) => {
                
            }
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _window_id: WindowId, _event: WindowEvent) {}
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

pub fn load_tray_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let decoded_image = image::load_from_memory(icon_bytes)?
        .to_rgba8();
    let (width, height) = decoded_image.dimensions();
    let raw_rgba_pixels = decoded_image.into_raw();
    let tray_icon = Icon::from_rgba(raw_rgba_pixels, width, height)?;
    Ok(tray_icon)
}