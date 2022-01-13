mod code;
mod parser;

use code::{boot, translate};
use parser::parse;
use std::env::args;
use std::path::Path;
use std::{fs, process};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;

fn main() -> Result<()> {
    if args().count() > 2 {
        eprintln!("Usage: vm-translator [file or directory]");
        process::exit(65);
    } else if let Some(arg) = args().nth(1) {
        if fs::metadata(&arg)?.is_dir() {
            translate_dir(&arg)?;
        } else {
            translate_file(&arg)?;
        }
    } else {
        todo!("run on cwd if called without args");
    }
    Ok(())
}

fn translate_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let dir_name = file_stem(path_ref)?;
    let mut contents = boot();
    for dir_entry in fs::read_dir(path_ref)? {
        let file_path = dir_entry?.path();
        match file_path.extension() {
            Some(ext) if ext == "vm" => {
                let stem = file_stem(&file_path)?;
                let source = fs::read_to_string(&file_path)?;
                contents += &translate_str(&source, stem)?;
            }
            _ => {}
        }
    }
    let out_path = path_ref.join(dir_name).with_extension("asm");
    fs::write(&out_path, contents)?;
    eprintln!("Successfully wrote {}", out_path.to_string_lossy());
    Ok(())
}

fn translate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let out_path = path_ref.with_extension("asm");
    let stem = file_stem(path_ref)?;
    let source = fs::read_to_string(path_ref)?;
    // boot must be excluded, manually for now, in order to translate earlier
    // VM scripts that didn't rely on calling Sys.init by convention.
    let contents = boot() + &translate_str(&source, stem)?;
    fs::write(&out_path, contents)?;
    eprintln!("Successfully wrote {}", out_path.to_string_lossy());
    Ok(())
}

fn translate_str(input: &str, static_prefix: &str) -> Result<String> {
    let (remaining_input, commands) = parse(input).unwrap_or_else(exit_with_error);
    if !remaining_input.is_empty() {
        eprintln!("[line {}] failed to parse entire input", commands.len());
        process::exit(65);
    }
    Ok(translate(&commands, static_prefix))
}

fn file_stem(path: &Path) -> Result<&str> {
    let stem = path
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or("invalid file name")?;
    Ok(stem)
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
