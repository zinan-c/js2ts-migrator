use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Error};

use crate::migrator;

static DRY_RUN: AtomicBool = AtomicBool::new(false);

pub fn set_dry_run(enabled: bool) {
    DRY_RUN.store(enabled, Ordering::SeqCst);
}

pub fn process_single_file(input_path: &Path, output_dir: &Path) -> Result<(), Error> {
    let result = (|| {
        if !is_supported_js(input_path) {
            return Err(Error::msg(
                "input file must have .js, .jsx, .mjs, or .cjs extension",
            ));
        }

        let source = fs::read_to_string(input_path)
            .with_context(|| format!("read input file {}", input_path.display()))?;
        let line_count = count_lines(&source);

        let module = migrator::parse_js_file(input_path)
            .with_context(|| format!("parse {}", input_path.display()))?;
        let type_map = migrator::infer_var_types(&module);
        let var_count = migrator::count_var_decls(&module);
        let content = migrator::generate_ts(module, &type_map);

        let out_path = output_path_for(input_path, output_dir)?;

        if DRY_RUN.load(Ordering::SeqCst) {
            return Ok(ProcessingStats {
                input_path: input_path.to_path_buf(),
                output_path: out_path,
                line_count,
                var_count,
                wrote_file: false,
            });
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create output dir {}", parent.display()))?;
        }
        fs::write(&out_path, content)
            .with_context(|| format!("write output file {}", out_path.display()))?;

        Ok(ProcessingStats {
            input_path: input_path.to_path_buf(),
            output_path: out_path,
            line_count,
            var_count,
            wrote_file: true,
        })
    })();

    match result {
        Ok(stats) => {
            let mode = if stats.wrote_file { "written" } else { "dry-run" };
            println!(
                "ok: {} -> {} ({mode}), vars: {}, lines: {}",
                stats.input_path.display(),
                stats.output_path.display(),
                stats.var_count,
                stats.line_count
            );
            Ok(())
        }
        Err(err) => {
            eprintln!("fail: {} ({})", input_path.display(), err);
            Err(err)
        }
    }
}

struct ProcessingStats {
    input_path: PathBuf,
    output_path: PathBuf,
    line_count: usize,
    var_count: usize,
    wrote_file: bool,
}

pub fn output_path_for(input_path: &Path, output_dir: &Path) -> Result<PathBuf, Error> {
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::msg("invalid file name"))?;

    let mut out_path = PathBuf::from(output_dir);
    out_path.push(format!("{file_stem}.ts"));
    Ok(out_path)
}

fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}

pub fn is_supported_js(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()).as_deref(),
        Some("js") | Some("jsx") | Some("mjs") | Some("cjs")
    )
}
