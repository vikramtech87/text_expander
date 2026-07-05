pub enum ExpansionSnippet {
    Text { content: String },
    Placeholder { name: String, default: Option<String> },
}