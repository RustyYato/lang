use std::path::PathBuf;

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
    println!("{:#?}", parser.parse_expr());

    Ok(())
}
