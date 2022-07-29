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
            Some("expected-parse" | "expected-parse-errors") => Ok(Self::ParseResult((
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

struct Counts {
    regened: u32,
    commited: u32,
}

trait Output {
    fn print(&self);
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

    let mut print_outputs = Vec::new();

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

    println!("\tFound {} test-case(s)...", tests.len().green());
    print!("\tRunning ui tests...");

    for (i, file) in tests.iter().enumerate() {
        if i % 50 == 0 {
            println!();
            print!("\t");
        }
        let mut errors = Vec::new();
        let mut parser = parser::parser::Parser::new(&mut errors, &file.test);

        let output: Box<dyn SerializeTest> = if file.test.starts_with("# parse_expr") {
            parser.consume_ignored_tokens();
            let output = parser.parse_expr();
            Box::new(output)
        } else {
            println!(
                "{}: unknown test kind for {} ({})",
                "WARNING".yellow(),
                file.name.yellow(),
                file.rel_path.display().dimmed()
            );
            continue;
        };

        print_outputs.extend(handle_test_case(
            &args,
            file,
            &output,
            "expected-parse",
            &results,
        )?);

        print_outputs.extend(handle_test_case(
            &args,
            file,
            &errors,
            "expected-parse-errors",
            &results,
        )?)
    }
    println!("\n");

    for out in print_outputs {
        out.print();
    }

    Ok(())
}

fn handle_test_case(
    args: &Args,
    file: &Test,
    output: &dyn SerializeTest,
    extension: &str,
    results: &HashMap<PathBuf, ParseResult>,
) -> anyhow::Result<Option<Box<dyn Output>>> {
    let output = output.to_serialize_string();
    let output = format(&output, SAVE_TAB);

    let expected_path = file.path.with_extension(extension);
    if let Some(expected_errors) = results.get(&expected_path) {
        let expected = format(&expected_errors.value, SAVE_TAB);
        if expected != output {
            struct FailedTest {
                message: String,
                output: String,
                expected: String,
            }

            impl Output for FailedTest {
                fn print(&self) {
                    println!("{}", self.message);
                    print_diff(&self.output, &self.expected);
                    println!();
                }
            }

            if args.regen {
                print!("{}", "R".magenta());

                std::fs::write(&expected_path, output)?;
            } else {
                print!("{}", "F".red());

                return Ok(Some(Box::new(FailedTest {
                    message: format!(
                        "failed test {} ({})",
                        file.name.bright_red(),
                        file.rel_path.display().dimmed(),
                    ),
                    output,
                    expected,
                })));
            }
        } else {
            print!("{}", ".".green());
        }
    } else if args.commit {
        print!("{}", "C".bright_yellow());
        std::fs::write(&expected_path, output)?;
    } else {
        print!("{}", "+".bright_yellow());
        struct NewTest {
            message: String,
            output: String,
        }

        impl Output for NewTest {
            fn print(&self) {
                let output = format(&self.output, TAB);
                println!("{}", self.message);
                println!("{}", output.yellow());
                println!();
            }
        }

        return Ok(Some(Box::new(NewTest {
            message: format!(
                "new {extension} test {} ({})",
                file.name.bright_yellow(),
                file.rel_path.display().dimmed()
            ),
            output,
        })));
    }
    Ok(None)
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
                let ends_are_ws = |s: &str| {
                    s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace)
                };

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
                                    if ends_are_ws(&expected) {
                                        print!("{}", expected.black().on_bright_red())
                                    } else {
                                        print!("{}", expected.bright_red())
                                    }
                                }
                                differ::Tag::Delete => {
                                    if ends_are_ws(&output) {
                                        print!("{}", output.black().on_bright_green())
                                    } else {
                                        print!("{}", output.bright_green())
                                    }
                                }
                                differ::Tag::Replace => {
                                    if ends_are_ws(&expected) {
                                        print!("{}", expected.black().on_bright_red())
                                    } else {
                                        print!("{}", expected.bright_red())
                                    }
                                    if ends_are_ws(&output) {
                                        print!("{}", output.black().on_bright_green())
                                    } else {
                                        print!("{}", output.bright_green())
                                    }
                                }
                            }
                        }

                        println!()
                    }
                } else {
                    for line in expected {
                        println!("\t{}", line.red())
                    }
                    for line in output {
                        println!("\t{}", line.green())
                    }
                }
            }
        }
    }
}
