use crate::{ExpansionRule, ExpansionSnippet};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParserError {
    InvalidNumberOfArguments(usize),
    FileAccessError(std::io::Error),
    CsvError(csv::Error),
    CsvInvalid,
}

impl From<std::io::Error> for ParserError {
    fn from(value: std::io::Error) -> Self {
        ParserError::FileAccessError(value)
    }
}

impl From<csv::Error> for ParserError {
    fn from(value: csv::Error) -> Self {
        ParserError::CsvError(value)
    }
}

pub fn tokenize_expansion(raw_text: &str) -> Result<Vec<ExpansionSnippet>, ParserError> {
    let mut snippets = Vec::new();
    let mut cursor = 0;
    let mut token_mode = false;

    while cursor < raw_text.len() {
        let reminder = &raw_text[cursor..];

        if !token_mode {
            let start = reminder.find("[[");
            match start {
                Some(start) => {
                    if start > 0 {
                        snippets.push(ExpansionSnippet::Text {
                            content: reminder[..start].to_string(),
                        });
                        cursor = cursor + start;
                    }
                    token_mode = true;
                },
                None => {
                    snippets.push(ExpansionSnippet::Text {
                        content: reminder.to_owned()
                    });
                    cursor = cursor + reminder.len();
                }
            }
        } else {
            let end = reminder.find("]]");
            match end {
                Some(end) => {
                    let token_str = &reminder[2..end];
                    let splits = &token_str.split_once(":");
                    match splits {
                        Some((name, default)) => {
                            snippets.push(ExpansionSnippet::Placeholder {
                                name: name.to_string(),
                                default: Some(default.to_string()),
                            })
                        }
                        None => {
                            snippets.push(ExpansionSnippet::Placeholder {
                                name: token_str.to_string(),
                                default: None,
                            })
                        }
                    }
                    cursor = cursor + end + 2;
                    token_mode = false;
                }
                None => {
                    return Err(ParserError::CsvInvalid);
                }
            }
        }
    }

    Ok(snippets)
}

fn snippet_to_string(snippet: &ExpansionSnippet) -> String {
    match &snippet {
        ExpansionSnippet::Placeholder { name, default: None } => {
            format!(r#"{{ type = "placeholder", name = "{}" }}"#, name)
        }
        ExpansionSnippet::Placeholder { name, default: Some(default) } => {
            format!(r#"{{ type = "placeholder", name = "{}", default = "{}" }}"#, name, default)
        }
        ExpansionSnippet::Text { content } => {
            format!(r#"{{ type = "text", content = "{}" }}"#, content)
        }
    }
}

pub fn rule_to_string(rule: &ExpansionRule) -> String {
    let mut output = format!("[[rules]]\ntrigger = \"{}\"", rule.trigger);
    if !rule.expansion.is_empty() {
        output.push_str("\nexpansion = [\n\t");
        let rules: Vec<String> = rule.expansion.iter().map(snippet_to_string).collect();
        output.push_str(&rules.join(",\n\t"));
        output.push_str("\n]")
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::ExpansionSnippet;
    use super::tokenize_expansion;

    #[test]
    fn test_pure_text_without_tokens() {
        let input = "Normal renal tissue.";
        let result = tokenize_expansion(input).unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            ExpansionSnippet::Text { content } => assert_eq!(content, "Normal renal tissue."),
            _ => panic!("Expected text snippet"),
        }
    }

    #[test]
    fn test_single_token_with_default() {
        let input = "[[count:6]] levels";
        let result = tokenize_expansion(input).unwrap();

        assert_eq!(result.len(), 2);
        match &result[0] {
            ExpansionSnippet::Placeholder { name, default } => {
                assert_eq!(name, "count");
                assert_eq!(default.as_deref(), Some("6"));
            }
            _ => panic!("Expected placeholder at index 0"),
        }
        match &result[1] {
            ExpansionSnippet::Text { content } => assert_eq!(content, " levels"),
            _ => panic!("Expected text snippet at index 1"),
        }
    }

    #[test]
    fn test_complex_mixed_string() {
        let input = "Examined at [[levels:6]] levels of [[type]].";
        let result = tokenize_expansion(input).unwrap();

        // Should break into: Text, Placeholder, Text, Placeholder, Text
        assert_eq!(result.len(), 5);
        match &result[0] {
            ExpansionSnippet::Text { content } => assert_eq!(content, "Examined at "),
            _ => panic!("Expected text snippet at index 0"),
        }
        match &result[1] {
            ExpansionSnippet::Placeholder { name, default } => {
                assert_eq!(name, "levels");
                assert_eq!(default.as_deref(), Some("6"));
            }
            _ => panic!("Expected placeholder at index 1"),
        }
        match &result[2] {
            ExpansionSnippet::Text { content } => assert_eq!(content, " levels of "),
            _ => panic!("Expected text snippet at index 2"),
        }
        match &result[3] {
            ExpansionSnippet::Placeholder { name, default } => {
                assert_eq!(name, "type");
                assert_eq!(default, &None);
            }
            _ => panic!("Expected placeholder at index 3"),
        }
        match &result[4] {
            ExpansionSnippet::Text { content } => assert_eq!(content, "."),
            _ => panic!("Expected text snippet at index 4"),
        }
    }

    #[test]
    fn test_invalid_string() {
        let input = "[[count:6 levels";
        let result = tokenize_expansion(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_scenario() {
        let input = "[[token_one]][[token_two]]";
        let result = tokenize_expansion(input).unwrap();
        assert_eq!(result.len(), 2);
    }
}