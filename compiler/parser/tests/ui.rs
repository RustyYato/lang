use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use parser::ast::SerializeTest;

use differ::Differ;
use owo_colors::OwoColorize;
use walkdir::WalkDir;

#[derive(Debug, clap::Parser)]
struct Args {
    #[clap(env)]
    regen: bool,
    #[clap(env)]
    commit: bool,
}

const TAB: &str = ".  ";
const SAVE_TAB: &str = "   ";

fn format(mut s: &str, tab: &str) -> String {
    let mut depth = 0;
    let mut output = String::new();
    let mut newline = false;

    let mut detected_tab = None;

    while let Some(index) = s.find(['(', '\n', ')', ',', '"']) {
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
            "\n" => match detected_tab {
                Some(detected_tab) => {
                    depth = 0;
                    while let Some(next) = s.strip_prefix(detected_tab) {
                        s = next;
                        depth += 1;
                    }
                    newline = true;
                    output.push('\n');
                }
                None => match s.find(|c: char| c.is_alphanumeric() || c == '"') {
                    Some(length) => {
                        let (tab, rest) = s.split_at(length);
                        detected_tab = Some(tab);
                        s = rest;
                        newline = true;
                        depth = 1;
                        output.push('\n');
                    }
                    None => {
                        output.push('\n');
                        break;
                    }
                },
            },
            _ => unreachable!(),
        }
    }

    output
}

enum File {
    Test(Test),
    ParseResult((PathBuf, ParseResult)),
}

struct Test {
    test: String,
    name: String,
    path: PathBuf,
    rel_path: PathBuf,
}

struct ParseResult {
    value: String,
}

impl File {
    fn load(root: &Path, path: &Path) -> anyhow::Result<Self> {
        let prefix = path.with_extension("");
        let name = prefix.file_stem();

        let name = match name {
            Some(name) => name.to_string_lossy().into_owned(),
            None => {
                return Err(anyhow::format_err!(
                    "cannot read test at {}",
                    path.display().bright_red()
                ));
            }
        };

        let ext = path.extension().and_then(OsStr::to_str);

        let read = || {
            std::fs::read_to_string(&path).map_err(|_| {
                anyhow::format_err!("could not open file {}", path.display().bright_red())
            })
        };

        match ext {
            Some("test") => Ok(Self::Test(Test {
                test: read()?,
                name,
                path: path.into(),
                rel_path: Path::new(".").join(path.strip_prefix(root)?),
            })),
            Some("expected-parse") => Ok(Self::ParseResult((
                path.into(),
                ParseResult { value: read()? },
            ))),
            Some(ext) => Err(anyhow::format_err!(
                "Invalid extension ({}) found for {}!",
                ext.bright_red(),
                path.display().bright_red()
            )),
            None => Err(anyhow::format_err!(
                "No extension found for {}!",
                path.display().bright_red()
            )),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args @ Args { regen, commit } = clap::Parser::parse();

    if regen {
        println!("Will regenerate failing tests...");
    }

    if commit {
        println!("Will commit new tests...");
    }

    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("ui");

    println!("\n\tCollecting ui tests...");

    let mut tests = Vec::new();
    let mut results = HashMap::new();

    for entry in WalkDir::new(&path) {
        let entry = entry?;
        let meta = entry.metadata()?;

        if !meta.is_file() {
            continue;
        }

        match File::load(&path, entry.path()) {
            Ok(File::Test(test)) => tests.push(test),
            Ok(File::ParseResult((path, result))) => {
                results.insert(path, result);
            }
            Err(err) => println!("{err}"),
        }
    }

    println!("\tFound {} test(s)...\n", tests.len().green());

    for file in &tests {
        let mut errors = Vec::new();
        let mut parser = parser::parser::Parser::new(&mut errors, &file.test);

        let output = if file.test.starts_with("# parse_expr") {
            parser.consume_ignored_tokens();
            let output = parser.parse_expr();
            output.to_serialize_string()
        } else {
            println!(
                "{}: unknown test kind for {} ({})",
                "WARNING".yellow(),
                file.name.yellow(),
                file.rel_path.display().dimmed()
            );
            continue;
        };

        let output = format(&output, SAVE_TAB);

        let expected_parse = file.path.with_extension("expected-parse");
        if let Some(result) = results.get(&expected_parse) {
            let expected = format(&result.value, SAVE_TAB);
            if expected != output {
                println!(
                    "failed test {} ({})",
                    file.name.bright_red(),
                    file.rel_path.display().dimmed()
                );
                print_diff(&output, &expected);
            }
        } else if args.commit {
            std::fs::write(&expected_parse, output)?;
        } else {
            let output = format(&output, TAB);
            println!(
                "new parse test {} ({})",
                file.name.bright_yellow(),
                file.rel_path.display().dimmed()
            );
            println!("{}", output.yellow());
        }

        if !errors.is_empty() {
            let output = errors.to_serialize_string();
            let expected_errors = file.path.with_extension("expected-parse-errors");
            if let Some(expected_errors) = results.get(&expected_errors) {
                let expected = format(&expected_errors.value, SAVE_TAB);
                if expected != output {
                    println!(
                        "failed test {} ({})",
                        file.name.bright_red(),
                        file.rel_path.display().dimmed()
                    );
                    print_diff(&output, &expected);
                }
            } else if args.commit {
                std::fs::write(&expected_errors, output)?;
            } else {
                let output = format(&output, TAB);
                println!(
                    "new errors test {} ({})",
                    file.name.bright_yellow(),
                    file.rel_path.display().dimmed()
                );
                println!("{}", output.yellow());
            }
        }
    }

    println!();

    Ok(())
}

fn print_diff(output: &str, expected: &str) {
    let output: Vec<_> = output.lines().collect();
    let expected: Vec<_> = expected.lines().collect();

    for span in Differ::new(&output, &expected).spans() {
        let output = &output[span.a_start..span.a_end];
        let expected = &expected[span.b_start..span.b_end];
        match span.tag {
            differ::Tag::Equal => {
                for line in output {
                    println!("\t{}", line.dimmed())
                }
            }
            differ::Tag::Insert => {
                for line in expected {
                    println!("\t{}", line.red())
                }
            }
            differ::Tag::Delete => {
                for line in output {
                    println!("\t{}", line.green())
                }
            }
            differ::Tag::Replace => {
                if output.len() == expected.len() {
                    for (&output, &expected) in output.iter().zip(expected) {
                        let output: Vec<_> = output.chars().collect();
                        let expected: Vec<_> = expected.chars().collect();

                        print!("\t");
                        for span in Differ::new(&output, &expected).spans() {
                            let output = String::from_iter(&output[span.a_start..span.a_end]);
                            let expected = String::from_iter(&expected[span.b_start..span.b_end]);

                            match span.tag {
                                differ::Tag::Equal => print!("{}", output.dimmed()),
                                differ::Tag::Insert => {
                                    print!("{}", expected.bright_red())
                                }
                                differ::Tag::Delete => {
                                    print!("{}", output.bright_green())
                                }
                                differ::Tag::Replace => {
                                    print!("{}", expected.bright_red());
                                    print!("{}", output.bright_green());
                                }
                            }
                        }

                        println!()
                    }
                } else {
                    for line in &output[span.a_start..span.a_end] {
                        println!("-\t{}", line.red())
                    }
                    for line in &expected[span.b_start..span.b_end] {
                        println!("+\t{}", line.green())
                    }
                }
            }
        }
    }
}
