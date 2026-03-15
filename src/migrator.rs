use std::collections::HashMap;
use std::path::Path;

use anyhow::Error;
use swc_common::{FilePathMapping, SourceMap, DUMMY_SP};
use swc_ecma_codegen::{text_writer::JsWriter, Config, Emitter};
use swc_ecma_parser::ast::{
    Expr, Lit, Module, Pat, TsArrayType, TsKeywordType, TsKeywordTypeKind, TsType, TsTypeAnn,
    VarDeclarator,
};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};
use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};

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
    let mut inferer = VarTypeInferer {
        types: HashMap::new(),
    };
    module.visit_with(&mut inferer);
    inferer.types
}

fn infer_type(expr: &Expr) -> Option<&'static str> {
    match expr {
        Expr::Lit(Lit::Str(_)) => Some("string"),
        Expr::Lit(Lit::Num(_)) => Some("number"),
        Expr::Lit(Lit::Bool(_)) => Some("boolean"),
        Expr::Tpl(_) => Some("string"),
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
    let mut annotator = TypeAnnotator { type_map };
    module.visit_mut_with(&mut annotator);
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

struct VarTypeInferer {
    types: HashMap<String, String>,
}

impl Visit for VarTypeInferer {
    fn visit_var_declarator(&mut self, decl: &VarDeclarator) {
        if let Pat::Ident(ident) = &decl.name {
            if let Some(init) = decl.init.as_deref() {
                if let Some(ty) = infer_type(init) {
                    self.types.insert(ident.id.sym.to_string(), ty.to_string());
                }
            }
        }

        decl.visit_children_with(self);
    }
}

struct TypeAnnotator<'a> {
    type_map: &'a HashMap<String, String>,
}

impl VisitMut for TypeAnnotator<'_> {
    fn visit_mut_var_declarator(&mut self, decl: &mut VarDeclarator) {
        decl.visit_mut_children_with(self);

        let Pat::Ident(ident) = &mut decl.name else {
            return;
        };

        if ident.type_ann.is_some() {
            return;
        }

        let name = ident.id.sym.to_string();
        let ty_name = self
            .type_map
            .get(&name)
            .map(|s| s.as_str())
            .or_else(|| decl.init.as_deref().and_then(infer_type))
            .unwrap_or("any");

        ident.type_ann = Some(Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type_from_name(ty_name)),
        }));
    }
}
