use crate::models::ExpansionSnippet;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct ExpansionRule {
    pub trigger: String,
    pub expansion: Vec<ExpansionSnippet>,
}