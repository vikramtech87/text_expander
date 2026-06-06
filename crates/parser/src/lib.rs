use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpansionSnippet {
    Text {
        content: String,
    },
    Placeholder {
        name: String,
        default: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpansionRule {
    pub trigger: String,
    pub expansion: Vec<ExpansionSnippet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rules: Vec<ExpansionRule>,
}

impl Config {
    pub fn parse(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(&toml_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let raw_toml = r#"
            [[rules]]
            trigger = ";gme"
            expansion = [
                { type = "text", content = "Hello " },
                { type = "placeholder", name = "username", default = "Friend" },
                { type = "text", content = "!" }
            ]
        "#;

        let config = Config::parse(raw_toml).unwrap();

        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].trigger, ";gme");

        match &config.rules[0].expansion[1] {
            ExpansionSnippet::Placeholder { name, .. } => assert_eq!(name, "username"),
            _ => panic!("Expected a placeholder snippet"),
        }
    }
}
