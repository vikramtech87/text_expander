use std::fmt;
use std::fmt::Formatter;
use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

impl ExpansionSnippet {
    pub fn is_placeholder(&self) -> bool {
        match self {
            ExpansionSnippet::Placeholder { .. } => true,
            _ => false,
        }
    }

    pub fn get_default(&self) -> Option<String> {
        match self {
            Self::Placeholder { default, name } => {
                Some(default.clone().unwrap_or_else(|| format!("[[{}]]", name)))
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_default() {
        let text = ExpansionSnippet::Text { content: "Something".to_string() };
        assert_eq!(text.get_default(), None);
        
        let placeholder = ExpansionSnippet::Placeholder { 
            name: "parenchyma".to_string(),
            default: None,
        };
        assert_eq!(placeholder.get_default(), Some("[[parenchyma]]".to_string()));
        
        let placeholder = ExpansionSnippet::Placeholder {
            name: "parenchyma".to_string(),
            default: Some("cortical".to_string()),
        };
        assert_eq!(placeholder.get_default(), Some("cortical".to_string()));
    }
}