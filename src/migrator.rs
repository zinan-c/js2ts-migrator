use std::collections::HashMap;
use std::path::Path;

use anyhow::Error;
use swc_common::{FileName, FilePathMapping, SourceMap, DUMMY_SP};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::ast::{
    Decl, Expr, Lit, Module, ModuleItem, Pat, Stmt, TsArrayType, TsKeywordType,
    TsKeywordTypeKind, TsType, TsTypeAnn,
};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};

pub fn parse_js_file(path: &Path) -> Result<Module, Error> {
    let cm = SourceMap::new(FilePathMapping::empty());
    let fm = cm.load_file(path)?;

    let syntax = Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    });
    let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);

    let module = parser.parse_module()?;
    Ok(module)
}

pub fn infer_var_types(module: &Module) -> HashMap<String, String> {
    let mut types = HashMap::new();

    for item in &module.body {
        let stmt = match item {
            ModuleItem::Stmt(stmt) => stmt,
            ModuleItem::ModuleDecl(_) => continue,
        };

        let decl = match stmt {
            Stmt::Decl(decl) => decl,
            _ => continue,
        };

        let var_decl = match decl {
            Decl::Var(var_decl) => var_decl,
            _ => continue,
        };

        for declarator in &var_decl.decls {
            let name = match &declarator.name {
                Pat::Ident(ident) => ident.id.sym.to_string(),
                _ => continue,
            };

            let Some(init) = declarator.init.as_deref() else {
                continue;
            };

            if let Some(ty) = infer_type(init) {
                types.insert(name, ty.to_string());
            }
        }
    }

    types
}

fn infer_type(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::Lit(Lit::Str(_)) => Some("string"),
        Expr::Lit(Lit::Num(_)) => Some("number"),
        Expr::Lit(Lit::Bool(_)) => Some("boolean"),
        Expr::Array(_) => Some("array"),
        Expr::Object(_) => Some("object"),
        _ => None,
    }
}

pub fn generate_ts(mut module: Module, type_map: &HashMap<String, String>) -> String {
    annotate_module(&mut module, type_map);

    let cm = SourceMap::new(FilePathMapping::empty());
    let mut buf = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: Config::default(),
            comments: None,
            cm: cm.clone(),
            wr: JsWriter::new(cm, "\n", &mut buf, None),
        };
        let _ = emitter.emit_module(&module);
    }

    String::from_utf8(buf).unwrap_or_default()
}

fn annotate_module(module: &mut Module, type_map: &HashMap<String, String>) {
    for item in &mut module.body {
        let stmt = match item {
            ModuleItem::Stmt(stmt) => stmt,
            ModuleItem::ModuleDecl(_) => continue,
        };

        let decl = match stmt {
            Stmt::Decl(decl) => decl,
            _ => continue,
        };

        let var_decl = match decl {
            Decl::Var(var_decl) => var_decl,
            _ => continue,
        };

        for declarator in &mut var_decl.decls {
            let ident = match &mut declarator.name {
                Pat::Ident(ident) => ident,
                _ => continue,
            };

            if ident.type_ann.is_some() {
                continue;
            }

            let ty_name = type_map
                .get(&ident.id.sym.to_string())
                .map(|s| s.as_str())
                .unwrap_or("any");

            ident.type_ann = Some(Box::new(TsTypeAnn {
                span: DUMMY_SP,
                type_ann: Box::new(ts_type_from_name(ty_name)),
            }));
        }
    }
}

fn ts_type_from_name(name: &str) -> TsType {
    match name {
        "string" => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        "number" => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNumberKeyword,
        }),
        "boolean" => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        }),
        "object" => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsObjectKeyword,
        }),
        "array" => TsType::TsArrayType(TsArrayType {
            span: DUMMY_SP,
            elem_type: Box::new(TsType::TsKeywordType(TsKeywordType {
                span: DUMMY_SP,
                kind: TsKeywordTypeKind::TsAnyKeyword,
            })),
        }),
        _ => TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsAnyKeyword,
        }),
    }
}
