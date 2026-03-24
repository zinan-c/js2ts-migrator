use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Error};
use walkdir::WalkDir;

use crate::file_processor;

pub fn run(input: &Path, output_dir: &Path, recursive: bool) -> Result<(), Error> {
    if input.is_file() {
        file_processor::process_single_file(input, output_dir)?;
        return Ok(());
    }

    if !input.is_dir() {
        return Err(Error::msg("input must be a file or directory"));
    }

    let mut success_count: usize = 0;
    let mut failure_count: usize = 0;

    if recursive {
        for entry in WalkDir::new(input).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && is_js(path) {
                let rel = path.strip_prefix(input).unwrap_or(path);
                let rel_dir = rel.parent().unwrap_or(Path::new(""));
                let out_dir = output_dir.join(rel_dir);
                match file_processor::process_single_file(path, &out_dir) {
                    Ok(()) => success_count += 1,
                    Err(_) => failure_count += 1,
                }
            }
        }
    } else {
        let mut seen_outputs: HashSet<String> = HashSet::new();
        for entry in fs::read_dir(input).with_context(|| {
            format!("read input directory {}", input.display())
        })? {
            let entry = entry.with_context(|| {
                format!("read directory entry in {}", input.display())
            })?;
            let path = entry.path();
            if path.is_file() && is_js(&path) {
                if let Ok(out_path) = file_processor::output_path_for(&path, output_dir) {
                    let key = out_path.display().to_string();
                    if !seen_outputs.insert(key.clone()) {
                        eprintln!(
                            "warn: output collision in non-recursive mode: {}",
                            key
                        );
                    }
                }
                match file_processor::process_single_file(&path, output_dir) {
                    Ok(()) => success_count += 1,
                    Err(_) => failure_count += 1,
                }
            }
        }
    }

    println!(
        "summary: {} succeeded, {} failed",
        success_count, failure_count
    );

    if failure_count > 0 {
        return Err(Error::msg(format!(
            "{} file(s) failed to process",
            failure_count
        )));
    }

    Ok(())
}

fn is_js(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()).as_deref(),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs")
    )
}
