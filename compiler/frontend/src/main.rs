use std::{fmt::write, path::PathBuf};

use parser::ast::SerializeTest;

use anyhow::Context;
use clap::Parser;

#[derive(Parser)]
pub struct Args {
    pub file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args: Args = Parser::parse();

    let file = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Could not open file: {}", args.file.display()))?;

    let mut parser = parser::parser::Parser::new(&file);

    parser.consume_ignored_tokens();
    let expr = parser.parse_expr();
    println!("{:#?}", expr);
    let expr = expr.to_serialize_string();
    println!("{}", expr);
    println!();
    println!("{}", format(&expr, ".  "));

    Ok(())
}

fn format(mut s: &str, tab: &str) -> String {
    let mut depth = 0;
    let mut output = String::new();
    let mut newline = false;

    while let Some(index) = s.find(['(', ')', ',', '"']) {
        let (before, after) = s.split_at(index);
        let before = before;
        let this = &after[..1];
        let after = &after[1..];
        s = after;

        if newline && (!before.is_empty() || this == "\"") {
            for _ in 0..depth {
                output.push_str(tab);
            }
        }
        output.push_str(before);

        match this {
            "(" => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                newline = true;
                depth += 1;
            }
            ")" => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                newline = true;
                depth -= 1
            }
            "," => {
                if !output.ends_with('\n') {
                    output.push('\n');
                }
                newline = true;
            }
            "\"" => {
                newline = false;
                let index = loop {
                    let index = s.find('"').unwrap();
                    if s.get(index.wrapping_sub(1)..index) != Some("\\") {
                        break index;
                    }
                };

                output.push('"');
                output.push_str(&s[..index + 1]);
                s = &s[index + 1..];
            }
            _ => unreachable!(),
        }
    }

    output
}
