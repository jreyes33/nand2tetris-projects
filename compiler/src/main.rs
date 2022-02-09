mod compiler;

use compiler::Compiler;
use std::env::args;
use std::path::Path;
use std::{fs, process};
use tree_sitter::{Parser, Tree};
use vm_translator::parser::Command;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn main() -> Result<()> {
    if args().count() > 2 {
        eprintln!("Usage: compiler [file or directory]");
        process::exit(65);
    } else if let Some(arg) = args().nth(1) {
        if fs::metadata(&arg)?.is_dir() {
            compile_dir(&arg)?;
        } else {
            compile_file(&arg)?;
        }
    } else {
        todo!("run on cwd if called without args");
    }
    Ok(())
}

fn compile_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let dir_name = file_stem(path_ref)?;
    let mut contents = String::new();
    for dir_entry in fs::read_dir(path_ref)? {
        let file_path = dir_entry?.path();
        match file_path.extension() {
            Some(ext) if ext == "jack" => {
                let stem = file_stem(&file_path)?;
                let source = fs::read_to_string(&file_path)?;
                contents += &compile_str(&source, stem)?;
            }
            _ => {}
        }
    }
    let out_path = path_ref.join(dir_name).with_extension("vm");
    fs::write(&out_path, contents)?;
    eprintln!("Successfully wrote {}", out_path.to_string_lossy());
    Ok(())
}

fn compile_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let out_path = path_ref.with_extension("vm");
    let stem = file_stem(path_ref)?;
    let source = fs::read_to_string(path_ref)?;
    let contents = compile_str(&source, stem)?;
    fs::write(&out_path, contents)?;
    eprintln!("Successfully wrote {}", out_path.to_string_lossy());
    Ok(())
}

fn compile_str(input: &str, class_name: &str) -> Result<String> {
    let tree = parse(input)?;
    let cursor = tree.walk();
    let mut compiler = Compiler::new(input, cursor);
    let commands = compiler.compile()?;
    let mut out = String::new();
    for cmd in commands {
        out += &command_to_str(cmd, class_name);
        out += "\n";
    }
    println!("\n========== vm commands ==========\n{}", out);
    Ok(out)
}

fn parse(input: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_jack::language())?;
    let tree = parser.parse(input, None).ok_or("failed to parse")?;
    Ok(tree)
}

fn command_to_str<'c>(command: &'c Command, class_name: &str) -> String {
    use Command::*;
    match command {
        Add => "add".to_string(),
        Function(name, param_count) => format!("function {class_name}.{name} {param_count}"),
        Call(name, arg_count) => format!("call {name} {arg_count}"),
        // TODO: implement Display for Segment.
        Push(segment, n) => format!("push {} {n}", &(format!("{segment:?}")).to_lowercase()),
        cmd => todo!("{cmd:?} in command_to_str"),
    }
}

fn file_stem(path: &Path) -> Result<&str> {
    let stem = path
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or("invalid file name")?;
    Ok(stem)
}

#[allow(unused)]
fn exit_with_error<V, E: std::error::Error>(e: E) -> V {
    eprintln!("{}", e);
    process::exit(65)
}
