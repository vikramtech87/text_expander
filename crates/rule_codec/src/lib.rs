use std::fs;
use std::path::Path;
use crate::models::{ExpansionRule, ExpansionSnippet};

pub mod models;
pub mod errors;

pub use errors::CodecError;
use crate::models::rules_config::RulesConfig;

pub fn serialize_rules(rules: &[ExpansionRule]) -> Vec<u8> {
    let mut buffer = Vec::new();

    for rule in rules {
        let trigger_bytes = rule.trigger.as_bytes();
        let trigger_len = trigger_bytes.len() as u32;
        let snippet_count = rule.expansion.len() as u32;

        buffer.extend_from_slice(&trigger_len.to_le_bytes());
        buffer.extend_from_slice(&snippet_count.to_le_bytes());
        buffer.extend_from_slice(trigger_bytes);

        for snippet in &rule.expansion {
            match snippet {
                ExpansionSnippet::Text { content} => {
                    let content_bytes = content.as_bytes();
                    let content_len = content_bytes.len() as u32;

                    buffer.push(0); // 0 - Indicate type of snippet: Text
                    buffer.extend_from_slice(&content_len.to_le_bytes());
                    buffer.extend_from_slice(content_bytes);
                }
                ExpansionSnippet::Placeholder { name, default } => {
                    let name_bytes = name.as_bytes();
                    let name_len = name_bytes.len() as u32;

                    match default {
                        None => {
                            buffer.push(1); // 1 - Indicate type of snippet
                            buffer.extend_from_slice(&name_len.to_le_bytes());
                            buffer.extend_from_slice(name_bytes);
                        }
                        Some(default) => {
                            let default_bytes = default.as_bytes();
                            let default_len = default_bytes.len() as u32;

                            buffer.push(2); // 2 - Type of snippet
                            buffer.extend_from_slice(&name_len.to_le_bytes());
                            buffer.extend_from_slice(name_bytes);
                            buffer.extend_from_slice(&default_len.to_le_bytes());
                            buffer.extend_from_slice(default_bytes);
                        }
                    }
                }
            }
        }
    }
    buffer
}

pub fn deserialize_rules(bytes: &[u8]) -> Result<Vec<ExpansionRule>, CodecError> {
    if bytes.len() <= 0 {
        return Ok(Vec::new());
    }

    let mut cursor = 0usize;
    let mut rules = Vec::new();
    while cursor < bytes.len() {
        let trigger_len = read_length(bytes, &mut cursor)?;
        let snippet_count = read_length(bytes, &mut cursor)?;
        let trigger = read_string(&bytes, &mut cursor, trigger_len as usize)?;

        let mut snippets: Vec<ExpansionSnippet> = Vec::new();
        for _ in 0..snippet_count {
            if cursor >= bytes.len() {
                return Err(CodecError::UnexpectedEof);
            }

            let tag = bytes[cursor];
            cursor += 1;

            match tag {
                0 => {
                    let content_len = read_length(bytes, &mut cursor)?;
                    let content = read_string(bytes, &mut cursor, content_len as usize)?;

                    snippets.push(ExpansionSnippet::Text { content });
                }
                1 => {
                    let name_len = read_length(bytes, &mut cursor)?;
                    let name = read_string(&bytes, &mut cursor, name_len as usize)?;

                    snippets.push(ExpansionSnippet::Placeholder { name, default: None });
                }
                2 => {
                    let name_len = read_length(bytes, &mut cursor)?;
                    let name = read_string(&bytes, &mut cursor, name_len as usize)?;

                    let default_len = read_length(bytes, &mut cursor)?;
                    let default = read_string(&bytes, &mut cursor, default_len as usize)?;

                    snippets.push(ExpansionSnippet::Placeholder { name, default: Some(default) });
                }
                unknown_tag => return Err(CodecError::InvalidTypeTag(unknown_tag)),
            }
        }
        rules.push(ExpansionRule {
            trigger,
            expansion: snippets,
        });
    }

    Ok(rules)
}

fn read_string(bytes: &[u8], cursor: &mut usize, len: usize) -> Result<String, CodecError> {
    let start = *cursor;
    if start + len > bytes.len() {
        return Err(CodecError::UnexpectedEof);
    }

    let slice = &bytes[start..start + len];
    *cursor += len;

    std::str::from_utf8(slice)
        .map_err(|_| CodecError::InvalidUtf8)
        .map(|s| s.to_string())
}

fn read_length(bytes: &[u8], cursor: &mut usize) -> Result<u32, CodecError> {
    let start = *cursor;
    if start + 4 > bytes.len() {
        return Err(CodecError::UnexpectedEof);
    }

    let len_bytes: [u8; 4] = bytes[start..start + 4]
        .try_into()
        .map_err(|_| CodecError::UnexpectedEof)?;

    *cursor += 4;
    Ok(u32::from_le_bytes(len_bytes))
}

pub fn load_legacy_toml(path: &Path) -> Result<RulesConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read legacy file: {}", e))?;

    toml::from_str(&content)
        .map_err(|e| format!("Failed to parse toml config: {}", e))
}

#[cfg(test)]
mod tests {
    use crate::models::ExpansionSnippet;
    use super::*;

    #[test]
    fn test_binary_roundtrip_complex_rule() {
        let rules = vec![ExpansionRule {
            trigger: r#"\path\to\trigger""#.to_string(),
            expansion: vec![
                ExpansionSnippet::Text {
                    content: "Line 1\nLine 2 with \"quotes\" and \\ backslashes.".to_string(),
                },
                ExpansionSnippet::Placeholder {
                    name: "user_name".to_string(),
                    default: None,
                },
                ExpansionSnippet::Placeholder {
                    name: "item_count".to_string(),
                    default: Some("0".to_string()),
                },
            ],
        }];

        let encoded = serialize_rules(&rules);
        let decoded = deserialize_rules(&encoded).unwrap();

        assert_eq!(rules, decoded);
    }
}