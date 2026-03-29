mod patterns;
mod scope;
mod statements;
mod types;

use fpas_parser::{Decl, Stmt, Visibility};
use std::collections::{HashMap, HashSet};

const PRIVATE_NAMESPACE_SEGMENT: &str = "__private__";

pub(super) fn declaration_name(decl: &Decl) -> &str {
    match decl {
        Decl::Const(c) => &c.name,
        Decl::Var(v) | Decl::MutableVar(v) => &v.name,
        Decl::TypeDef(td) => &td.name,
        Decl::Function(f) => &f.name,
        Decl::Procedure(p) => &p.name,
    }
}

/// Implements qualified user-unit names and private-unit encapsulation from
/// `docs/pascal/09-units.md`.
pub(super) fn linked_decl_name(
    unit_name: &str,
    short_name: &str,
    visibility: Visibility,
) -> String {
    match visibility {
        Visibility::Public => format!("{unit_name}.{short_name}"),
        Visibility::Private => {
            format!("{unit_name}.{PRIVATE_NAMESPACE_SEGMENT}.{short_name}")
        }
    }
}

pub(super) fn rename_top_level_decls(decls: &mut [Decl], unit_name: &str) {
    for decl in decls {
        let old_name = declaration_name(decl).to_string();
        let new_name = linked_decl_name(unit_name, &old_name, decl.visibility());
        declaration_name_mut(decl).clone_from(&new_name);
    }
}

fn declaration_name_mut(decl: &mut Decl) -> &mut String {
    match decl {
        Decl::Const(c) => &mut c.name,
        Decl::Var(v) | Decl::MutableVar(v) => &mut v.name,
        Decl::TypeDef(td) => &mut td.name,
        Decl::Function(f) => &mut f.name,
        Decl::Procedure(p) => &mut p.name,
    }
}

pub(super) struct NameRewriter<'a> {
    path: String,
    resolved: &'a HashMap<String, String>,
    ambiguous: &'a HashMap<String, Vec<String>>,
    value_scopes: Vec<HashSet<String>>,
    type_scopes: Vec<HashSet<String>>,
    first_error: Option<String>,
}

impl<'a> NameRewriter<'a> {
    pub(super) fn new(
        path: String,
        resolved: &'a HashMap<String, String>,
        ambiguous: &'a HashMap<String, Vec<String>>,
    ) -> Self {
        Self {
            path,
            resolved,
            ambiguous,
            value_scopes: vec![HashSet::new()],
            type_scopes: vec![HashSet::new()],
            first_error: None,
        }
    }

    pub(super) fn raise_first_error(self) -> Result<(), String> {
        match self.first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub(super) fn rewrite_declarations(&mut self, decls: &mut [Decl]) {
        for decl in decls.iter() {
            self.predeclare_decl_name(decl);
        }
        for decl in decls.iter_mut() {
            self.rewrite_decl(decl);
        }
    }

    pub(super) fn rewrite_statements(&mut self, stmts: &mut [Stmt]) {
        for stmt in stmts {
            self.rewrite_stmt(stmt);
        }
    }
}
