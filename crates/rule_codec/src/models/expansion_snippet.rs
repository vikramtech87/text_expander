use std::fmt;
use std::fmt::Formatter;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpansionSnippet {
    Text { content: String },
    Placeholder { 
        name: String,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String> 
    },
}

impl fmt::Display for ExpansionSnippet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ExpansionSnippet::Placeholder { name, default: None } => {
                write!(f, "{{{{{}}}}}", name)
            }
            ExpansionSnippet::Placeholder { name, default: Some(default) } => {
                write!(f, "{{{{{}={}}}}}", name, default)
            }
            ExpansionSnippet::Text { content } => {
                write!(f, "{}", content)
            }
        }
    }
}