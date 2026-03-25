use std::path::{Path, PathBuf};

use crate::file_processor::{output_path_for, is_supported_js};
use crate::migrator::{count_var_decls, generate_ts, infer_var_types, parse_js_source};

#[test]
fn infer_types_and_count_vars() {
    let source = "const a = 1; let b = 'x'; var c = true; const {d} = obj;";
    let module = parse_js_source(source, "test.js").expect("parse");
    let types = infer_var_types(&module);
    assert_eq!(types.get("a").map(String::as_str), Some("number"));
    assert_eq!(types.get("b").map(String::as_str), Some("string"));
    assert_eq!(types.get("c").map(String::as_str), Some("boolean"));
    assert_eq!(count_var_decls(&module), 3);
}

#[test]
fn generate_ts_adds_type_annotations() {
    let source = "let a = 1; const b = 'x';";
    let module = parse_js_source(source, "test.js").expect("parse");
    let types = infer_var_types(&module);
    let output = generate_ts(module, &types);
    assert!(output.contains("let a: number = 1"));
    assert!(output.contains("const b: string ="));
}

#[test]
fn output_path_uses_ts_extension() {
    let input = PathBuf::from("example.mjs");
    let output_dir = PathBuf::from("out");
    let out = output_path_for(&input, &output_dir).expect("path");
    assert!(out.ends_with("example.ts"));
}

#[test]
fn supports_multiple_extensions() {
    let cases = ["a.js", "b.jsx", "c.mjs", "d.cjs"];
    for case in cases {
        assert!(is_supported_js(Path::new(case)));
    }
    assert!(!is_supported_js(Path::new("e.ts")));
}
