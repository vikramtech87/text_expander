use crate::models::ExpansionRule;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RulesConfig {
    pub rules: Vec<ExpansionRule>,
}