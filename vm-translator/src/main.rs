mod code;
mod parser;

use code::translate;
use parser::parse;
use std::env::args;
use std::fmt::Write;
use std::path::Path;
use std::{fs, process};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn main() -> Result<()> {
    if args().count() != 2 {
        println!("Usage: vm-translator [file]");
    } else if let Some(arg) = args().nth(1) {
        translate_file(arg)?;
    }
    Ok(())
}

fn translate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let out_path = path_ref.with_extension("asm");
    let file_name = path_ref
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or("invalid file name")?;
    let source = fs::read_to_string(path_ref)?;
    fs::write(out_path, translate_str(&source, file_name)?)?;
    Ok(())
}

fn translate_str(input: &str, file_name: &str) -> Result<String> {
    let (remaining_input, commands) = parse(input).unwrap_or_else(exit_with_error);
    if !remaining_input.is_empty() {
        eprintln!("[line {}] failed to parse entire input", commands.len());
        process::exit(65);
    }
    let mut output = String::new();
    for command in commands.iter().flatten() {
        writeln!(output, "{}", translate(command, file_name))?;
    }
    Ok(output)
}

fn exit_with_error<V, E: std::error::Error>(e: E) -> V {
    eprintln!("{}", e);
    process::exit(65)
}

#[cfg(test)]
mod tests {
    use super::translate_str;
    #[test]
    fn test_add() {
        translate_str(
            "push constant 7
             push constant 8
             add",
            "Foo",
        )
        .unwrap();
    }
}
