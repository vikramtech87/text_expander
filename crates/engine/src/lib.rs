use parser::{Config, ExpansionSnippet};

pub struct Engine {
    config: Config,
    buffer: String,
    max_buffer_len: usize,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        let max_trigger_len = config
            .rules
            .iter()
            .map(|rule| rule.trigger.len())
            .max()
            .unwrap_or(0);

        let max_buffer_len = std::cmp::max(max_trigger_len * 2, 20);

        Self {
            config,
            buffer: String::new(),
            max_buffer_len,
        }
    }

    pub fn push_char(&mut self, ch: char) -> Option<(Vec<ExpansionSnippet>, usize)> {
        self.buffer.push(ch);

        // Enforce the rolling buffer constraint: remove old characters from the front
        if self.buffer.len() > self.max_buffer_len {
            // Find the character boundary to safely remove the first character
            if let Some((idx, _)) = self.buffer.char_indices().nth(1) {
                self.buffer.drain(..idx);
            }
        }

        for rule in &self.config.rules {
            if self.buffer.ends_with(&rule.trigger) {
                let trigger_len = rule.trigger.chars().count();

                self.buffer.clear();
                return Some((rule.expansion.clone(), trigger_len));
            }
        }

        None
    }

    pub fn handle_backspace(&mut self) {
        self.buffer.pop();
    }

    pub fn update_config(&mut self, new_config: Config) {
        self.config = new_config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::{ExpansionRule, ExpansionSnippet};

    fn create_mock_engine() -> Engine {
        let config = Config {
            rules: vec![ExpansionRule {
                trigger: ";brb".to_string(),
                expansion: vec![ExpansionSnippet::Text {
                    content: "Be right back!".to_string(),
                }]
            }]
        };
        Engine::new(config)
    }

    #[test]
    fn test_match_trigger() {
        let mut engine = create_mock_engine();

        assert_eq!(engine.push_char('H'), None);
        assert_eq!(engine.push_char('i'), None);
        assert_eq!(engine.push_char(';'), None);
        assert_eq!(engine.push_char('b'), None);
        assert_eq!(engine.push_char('r'), None);
        let result = engine.push_char('b');
        assert!(result.is_some());

        if let Some((snippets, trigger_len)) = result {
            match &snippets[0] {
                ExpansionSnippet::Text { content} => {
                    assert_eq!(content, "Be right back!");
                },
                _ => panic!("Unexpected snippet type"),
            }
            assert_eq!(trigger_len, 4);
        }
    }

    #[test]
    fn test_backspace_handling() {
        let mut engine = create_mock_engine();

        engine.push_char(';');
        engine.push_char('b');
        engine.push_char('x');
        engine.handle_backspace();
        engine.push_char('r');
        let result = engine.push_char('b');
        assert!(result.is_some(), "Engine failed to match after a backspace correction");
    }
}