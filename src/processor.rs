use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::migrator;
use walkdir::WalkDir;

pub fn run(input: &Path, output_dir: &Path, recursive: bool) -> io::Result<()> {
    if input.is_file() {
        process_file(input, output_dir, Path::new(""))?;
        return Ok(());
    }

    if !input.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input must be a file or directory",
        ));
    }

    if recursive {
        for entry in WalkDir::new(input).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && is_js(path) {
                let rel = path.strip_prefix(input).unwrap_or(path);
                process_file(path, output_dir, rel.parent().unwrap_or(Path::new("")))?;
            }
        }
    } else {
        for entry in fs::read_dir(input)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && is_js(&path) {
                process_file(&path, output_dir, Path::new(""))?;
            }
        }
    }

    Ok(())
}

fn process_file(input_file: &Path, output_dir: &Path, relative_dir: &Path) -> io::Result<()> {
    if !is_js(input_file) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input file must have .js extension",
        ));
    }

    let module = migrator::parse_js_file(input_file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let type_map = migrator::infer_var_types(&module);
    let content = migrator::generate_ts(module, &type_map);

    let file_stem = input_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?;

    let mut out_path = PathBuf::from(output_dir);
    if !relative_dir.as_os_str().is_empty() {
        out_path.push(relative_dir);
    }
    fs::create_dir_all(&out_path)?;
    out_path.push(format!("{file_stem}.ts"));

    fs::write(&out_path, content)?;
    Ok(())
}

fn is_js(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("js"))
        .unwrap_or(false)
}
