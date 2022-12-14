use std::path::PathBuf;

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

    let mut errors = Vec::new();
    let mut parser = parser::parser::Parser::new(&mut errors, &file);

    parser.consume_ignored_tokens();
    let file = parser.parse_file();
    println!("{:#?}", file);
    let file = file.to_serialize_string();
    println!("{}", file);
    println!();

    Ok(())
}
