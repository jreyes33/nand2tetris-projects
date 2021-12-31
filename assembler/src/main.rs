mod code;
mod error;
mod instruction;
mod parser;
mod scanner;
mod token;

use code::Generator;
use error::Error;
use parser::Parser;
use scanner::Scanner;
use std::env::args;
use std::fmt::Write;
use std::path::Path;
use std::{fs, process};

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

fn main() -> Result {
    if args().count() != 2 {
        println!("Usage: assembler [file]");
    } else if let Some(arg) = args().nth(1) {
        assemble_file(arg)?;
    }
    Ok(())
}

fn assemble_file<P: AsRef<Path>>(path: P) -> Result {
    let out_path = path.as_ref().with_extension("hack");
    let source = fs::read_to_string(path)?;
    let mut scanner = Scanner::new(&source);
    let mut parser = Parser::new(scanner.scan_tokens());
    let instructions = parser.parse().unwrap_or_else(exit_with_error);
    let mut generator = Generator::new(&instructions);
    generator.register_labels().unwrap_or_else(exit_with_error);
    let mut output = String::new();
    for line in generator {
        writeln!(output, "{:0>16b}", line)?;
    }
    fs::write(out_path, output)?;
    Ok(())
}

fn exit_with_error<V>(e: Error) -> V {
    eprintln!("{}", e);
    process::exit(65)
}
