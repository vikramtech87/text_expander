mod utils;

use crate::utils::{rule_to_string, ParserError};
use csv::ReaderBuilder;
use parser::ExpansionRule;
use std::env;
use std::fs::File;

fn main() -> Result<(), ParserError> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(ParserError::InvalidNumberOfArguments(args.len()));
    }

    let file_name = &args[1];
    let file = File::open(file_name)?;

    let mut reader = ReaderBuilder::new().has_headers(false).from_reader(file);

    let mut compiled_rules = Vec::new();
    for record_result in reader.records() {
        let record = record_result.map_err(|_| ParserError::CsvInvalid)?;
        if record.len() != 2 {
            return Err(ParserError::CsvInvalid);
        }

        let trigger = record[0].to_owned();
        let expansion = &record[1];

        let snippets = utils::tokenize_expansion(expansion)?;

        let rule = ExpansionRule {
            trigger,
            expansion: snippets,
        };

        compiled_rules.push(rule);
    }

    // let output_config = Config {
    //     rules: compiled_rules,
    // };

    // match toml::to_string_pretty(&output_config) {
    //     Ok(toml_text) => {
    //         println!("\n-- 📋 COMPILED TOML RULES START ---");
    //         println!("{}", toml_text);
    //         println!("--- 📋 COMPILED TOML RULES END ---\n");
    //         println!("🚀 Success! Copy the block above and paste it into your rules.toml.");
    //     },
    //     Err(_) => return Err(ParserError::CsvInvalid),
    // }

    let output: Vec<String> = compiled_rules.iter().map(rule_to_string).collect();

    println!("\n-- 📋 COMPILED TOML RULES START ---");
    println!("{}", output.join("\n\n"));
    println!("--- 📋 COMPILED TOML RULES END ---\n");
    println!("🚀 Success! Copy the block above and paste it into your rules.toml.");
    Ok(())
}
