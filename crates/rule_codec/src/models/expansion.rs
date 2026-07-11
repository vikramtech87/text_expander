use std::fmt::Formatter;
use std::str::FromStr;
use crate::errors::ParseExpansionError;
use crate::models::ExpansionSnippet;

#[derive(Debug, PartialEq)]
pub struct Expansion(pub Vec<ExpansionSnippet>);

impl FromStr for Expansion {
    type Err = ParseExpansionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cursor = 0usize;
        let mut snippets = Vec::<ExpansionSnippet>::new();

        while cursor < s.len() {
            let open_brace_pos = s[cursor..].find("{{");
            match open_brace_pos {
                Some(pos) => {
                    // 1. Capture the text snippet before placeholder starts
                    if pos > 0 {
                        let text_snippet = ExpansionSnippet::Text {
                            content: String::from(&s[cursor..cursor + pos]),
                        };
                        snippets.push(text_snippet);
                    }

                    // 2. Move the cursor to after the {{
                    cursor += pos + 2;

                    // 3. Parse the placeholder
                    let placeholder = parse_placeholder(s, &mut cursor)?;
                    snippets.push(placeholder);

                }
                None => {
                    let text_snippet = ExpansionSnippet::Text {
                        content: String::from(&s[cursor..])
                    };
                    snippets.push(text_snippet);
                    cursor = s.len();
                }
            }
        }


        Ok(Self(snippets))
    }
}

fn parse_placeholder(s: &str, cursor: &mut usize) -> Result<ExpansionSnippet, ParseExpansionError> {
    let start = *cursor;
    let placeholder_close_pos = s[start..]
        .find("}}")
        .ok_or_else(|| ParseExpansionError::MalformedPlaceholder(s.to_string()))?;

    let content = &s[start..start+placeholder_close_pos];

    // To detect empty placeholders {{}}
    if content.trim() == "" {
        return Err(ParseExpansionError::MalformedPlaceholder(s.to_string()));
    }

    // Split at =
    let parts = content.split("=").collect::<Vec<_>>();

    // To detect more than one =
    if parts.len() > 2 {
        return Err(ParseExpansionError::MalformedPlaceholder(s.to_string()));
    }

    // Move the cursor to after }}
    *cursor = start + placeholder_close_pos + 2;

    if parts.len() == 1 {
        let name = parts[0].trim().to_string();
        if name.len() == 0 {
            return Err(ParseExpansionError::MalformedPlaceholder(s.to_string()));
        }
        return Ok(ExpansionSnippet::Placeholder {
            name,
            default: None,
        })
    }

    let name = parts[0].trim().to_string();
    let default = parts[1].trim().to_string();

    if name.len() == 0 || default.len() == 0 {
        return Err(ParseExpansionError::MalformedPlaceholder(s.to_string()));
    }

    Ok(ExpansionSnippet::Placeholder {
        name,
        default: Some(default),
    })
}

impl std::fmt::Display for Expansion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for snippet in &self.0 {
            write!(f, "{}", snippet)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_happy_path() {
        let input = "Copying {{ item = file.txt }} to {{ target }}.";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![
            ExpansionSnippet::Text { content: "Copying ".to_string() },
            ExpansionSnippet::Placeholder {
                name: "item".to_string(),
                default: Some("file.txt".to_string()),
            },
            ExpansionSnippet::Text { content: " to ".to_string() },
            ExpansionSnippet::Placeholder {
                name: "target".to_string(),
                default: None,
            },
            ExpansionSnippet::Text { content: ".".to_string() },
        ]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_from_str_handles_internal_spaces() {
        // Spaces inside the brackets should be cleanly trimmed out
        let input = "{{   variable   =   default value   }}";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![ExpansionSnippet::Placeholder {
            name: "variable".to_string(),
            default: Some("default value".to_string()),
        }]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_from_str_missing_closing_brackets() {
        let input = "Hello {{ user broken string";
        let result = Expansion::from_str(input);

        assert!(result.is_err(), "Should fail when missing closing brackets '}}'");
    }

    #[test]
    fn test_from_str_empty_placeholder_name() {
        let input = "Hello {{}}";
        let result = Expansion::from_str(input);
        assert!(result.is_err(), "Should fail if the placeholder name is empty");

        let input_with_equals = "Hello {{ = default }}";
        let result_with_equals = Expansion::from_str(input_with_equals);
        assert!(result_with_equals.is_err(), "Should fail if name before '=' is empty");
    }

    #[test]
    fn test_multiple_placeholders_with_intervening_text() {
        // This tests if your cursor calculation holds up across multiple transitions
        let input = "{{first}} middle text {{second}}";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![
            ExpansionSnippet::Placeholder { name: "first".to_string(), default: None },
            ExpansionSnippet::Text { content: " middle text ".to_string() },
            ExpansionSnippet::Placeholder { name: "second".to_string(), default: None },
        ]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_placeholder_at_absolute_end() {
        // This tests if the loop terminates cleanly when a placeholder ends exactly at s.len()
        let input = "Text then {{ending}}";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![
            ExpansionSnippet::Text { content: "Text then ".to_string() },
            ExpansionSnippet::Placeholder { name: "ending".to_string(), default: None },
        ]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_empty_spaces_only_placeholder_name_fails() {
        let input = "Hello {{   = default }}";
        let result = Expansion::from_str(input);
        assert!(result.is_err(), "Should catch blank space names preceding assignment operator");
    }

    #[test]
    fn test_from_str_with_multi_line_text() {
        // Multi-line raw string literals contain actual \n characters
        let input = "First Line\nSecond Line with {{ variable }}\nThird Line.";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![
            ExpansionSnippet::Text { content: "First Line\nSecond Line with ".to_string() },
            ExpansionSnippet::Placeholder { name: "variable".to_string(), default: None },
            ExpansionSnippet::Text { content: "\nThird Line.".to_string() },
        ]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_display_with_multi_line_text() {
        let expansion = Expansion(vec![
            ExpansionSnippet::Text { content: "Line 1\nLine 2\n".to_string() },
            ExpansionSnippet::Placeholder { name: "item".to_string(), default: None },
        ]);

        assert_eq!(expansion.to_string(), "Line 1\nLine 2\n{{item}}");
    }

    #[test]
    fn test_from_str_with_newline_in_default_value() {
        // Trimming rules should preserve internal newlines but clean outer space
        let input = "{{ snippet = Header\n-------\nBody text }}";
        let parsed = Expansion::from_str(input).unwrap();

        let expected = Expansion(vec![ExpansionSnippet::Placeholder {
            name: "snippet".to_string(),
            default: Some("Header\n-------\nBody text".to_string()),
        }]);

        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_display_empty_expansion() {
        let expansion = Expansion(vec![]);
        assert_eq!(expansion.to_string(), "");
    }

    #[test]
    fn test_display_pure_text_snippet() {
        let expansion = Expansion(vec![
            ExpansionSnippet::Text { content: "Simple plain string value.".to_string() }
        ]);
        assert_eq!(expansion.to_string(), "Simple plain string value.");
    }

    #[test]
    fn test_display_placeholder_without_default() {
        let expansion = Expansion(vec![
            ExpansionSnippet::Placeholder { name: "username".to_string(), default: None }
        ]);
        assert_eq!(expansion.to_string(), "{{username}}");
    }

    #[test]
    fn test_display_placeholder_with_default() {
        let expansion = Expansion(vec![
            ExpansionSnippet::Placeholder {
                name: "counter".to_string(),
                default: Some("1".to_string())
            }
        ]);
        assert_eq!(expansion.to_string(), "{{counter=1}}");
    }

    #[test]
    fn test_display_complex_interleaved_mix() {
        let expansion = Expansion(vec![
            ExpansionSnippet::Text { content: "Action: ".to_string() },
            ExpansionSnippet::Placeholder { name: "verb".to_string(), default: None },
            ExpansionSnippet::Text { content: " the file from ".to_string() },
            ExpansionSnippet::Placeholder {
                name: "source".to_string(),
                default: Some("src/".to_string())
            },
            ExpansionSnippet::Text { content: " now.".to_string() },
        ]);

        assert_eq!(
            expansion.to_string(),
            "Action: {{verb}} the file from {{source=src/}} now."
        );
    }
}