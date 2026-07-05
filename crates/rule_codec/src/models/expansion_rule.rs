use crate::models::ExpansionSnippet;

pub struct ExpansionRule {
    trigger: String,
    expansion: Vec<ExpansionSnippet>,
}