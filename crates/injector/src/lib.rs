use arboard::Clipboard;
use enigo::{Enigo, Keyboard, Settings, Key, Direction};
use std::thread;
use std::time::Duration;

pub struct Injector {
    enigo: Enigo,
    clipboard: Clipboard,
}

impl Injector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            enigo: Enigo::new(&Settings::default())?,
            clipboard: Clipboard::new()?,
        })
    }

    pub fn delete_chars(&mut self, count: usize) {
        for _ in 0..count {
            let _ = self.enigo.key(Key::Backspace, Direction::Click);
            // Give the OS a tiny fraction of a millisecond to process the key
            thread::sleep(Duration::from_millis(2));
        }
        // Small extra pause to ensure all backspaces are cleared by the OS
        thread::sleep(Duration::from_millis(10));
    }

    pub fn inject_text(&mut self, txt: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 0. Normalize the text for legacy windows
        let normalized_text = if cfg!(target_os = "windows") {
            txt.replace("\r\n", "\n").replace("\n", "\r\n")
        } else {
            txt.to_string()
        };

        // 1. Save the user's current clipboard content so we don't ruin their copy-paste state
        let previous_clipboard = self.clipboard.get_text().ok();

        // 2. Set the clipboard to our expanded text
        self.clipboard.set_text(normalized_text.clone())?;

        // 3. HARDENED FIX: Actively poll the clipboard until the OS confirms
        // the text change is live. Timeout after 150ms.
        let mut verified = false;
        for _ in 0..15 {
            if let Ok(curr) = self.clipboard.get_text() {
                if curr == normalized_text {
                    verified = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        if !verified {
            return Err("OS Clipboard failed to register expanded text in time".into());
        }

        #[cfg(target_os = "macos")]
        {
            let _ = self.enigo.key(Key::Meta, Direction::Press);
            thread::sleep(Duration::from_millis(5));
            let _ = self.enigo.key(Key::Unicode('v'), Direction::Click);
            thread::sleep(Duration::from_millis(5));
            let _ = self.enigo.key(Key::Meta, Direction::Release);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = self.enigo.key(Key::Control, Direction::Press);
            thread::sleep(Duration::from_millis(5));
            let _ = self.enigo.key(Key::Unicode('v'), Direction::Click);
            thread::sleep(Duration::from_millis(5));
            let _ = self.enigo.key(Key::Control, Direction::Release);
        }

        // #[cfg(target_os = "windows")]
        // {
        //     use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_PASTE};
        //
        //     unsafe {
        //         // Find the window currently active on the user's screen
        //         let hwnd = GetForegroundWindow();
        //         if hwnd != std::ptr::null_mut() {
        //             // Send WM_PASTE (0x0302) directly to the window control buffer.
        //             // This forces the app to paste immediately without using keyboard combinations.
        //             SendMessageW(hwnd, WM_PASTE, 0, 0);
        //         } else {
        //             return Err("No active window found".into());
        //         }
        //     }
        // }

        // 5. HARDENED FIX FOR LEAKS: Give the target application 250ms to
        // read the clipboard. 150ms is often too short if the PC is under heavy load.
        thread::sleep(Duration::from_millis(250));

        if let Some(old_text) = previous_clipboard {
            let _ = self.clipboard.set_text(old_text);
        }

        Ok(())
    }

    pub fn select_chars_backward(&mut self, count: usize) {
        let _ = self.enigo.key(Key::Shift, Direction::Press);
        for _ in 0..count {
            let _ = self.enigo.key(Key::LeftArrow, Direction::Click);
            thread::sleep(Duration::from_millis(2));
        }
        let _ = self.enigo.key(Key::Shift, Direction::Release);
    }
}