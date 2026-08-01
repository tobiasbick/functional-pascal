//! Hierarchical symbol extraction from recovered parser ASTs.

mod members;
mod routines;
mod source;

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Span;
use fpas_parser::{CompilationUnit, Decl, TypeBody, Visibility};

use super::{DocumentSymbol, SymbolKind, SymbolVisibility};
use crate::DocumentSnapshot;

pub(crate) use routines::{function_children, procedure_children};
pub(crate) use source::{
    function_detail, function_signature, name_span, named_type, procedure_detail,
    procedure_signature, type_callable_signature, type_text,
};

pub(super) fn extract(snapshot: &DocumentSnapshot) -> (String, Vec<DocumentSymbol>) {
    match snapshot.compilation_unit() {
        CompilationUnit::Program(program) => {
            let owner = program.name.clone();
            let full_span = SourceSpan::from(program.span);
            let mut children = program
                .declarations
                .iter()
                .map(|declaration| declaration_symbol(snapshot, &owner, declaration, full_span))
                .collect::<Vec<_>>();
            routines::collect_statement_symbols(
                snapshot,
                &owner,
                &program.body,
                full_span,
                &mut children,
            );
            let entries = vec![DocumentSymbol {
                name: program.name.clone(),
                qualified_name: program.name.clone(),
                kind: SymbolKind::Program,
                full_span,
                selection_span: program.name_span.into(),
                scope_span: full_span,
                visible_from: 0,
                visibility: SymbolVisibility::Private,
                type_name: None,
                detail: format!("program {}", program.name),
                callable: None,
                children,
            }];
            (owner, entries)
        }
        CompilationUnit::Unit(unit) => {
            let owner = unit.name.parts.join(".");
            let full_span = SourceSpan::from(unit.span);
            let children = unit
                .declarations
                .iter()
                .map(|declaration| declaration_symbol(snapshot, &owner, declaration, full_span))
                .collect();
            let entries = vec![DocumentSymbol {
                name: owner.clone(),
                qualified_name: owner.clone(),
                kind: SymbolKind::Unit,
                full_span,
                selection_span: unit.name.span.into(),
                scope_span: full_span,
                visible_from: 0,
                visibility: SymbolVisibility::Public,
                type_name: None,
                detail: format!("unit {owner}"),
                callable: None,
                children,
            }];
            (owner, entries)
        }
    }
}

pub(super) fn declaration_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &Decl,
    scope_span: SourceSpan,
) -> DocumentSymbol {
    let (name, kind, span, decl_visibility, type_name, detail) = match declaration {
        Decl::Const(value) => (
            &value.name,
            SymbolKind::Constant,
            value.span,
            value.visibility,
            named_type(&value.type_expr),
            format!(
                "const {}: {}",
                value.name,
                type_text(snapshot, &value.type_expr)
            ),
        ),
        Decl::Var(value) => (
            &value.name,
            SymbolKind::Variable,
            value.span,
            value.visibility,
            named_type(&value.type_expr),
            format!(
                "var {}: {}",
                value.name,
                type_text(snapshot, &value.type_expr)
            ),
        ),
        Decl::MutableVar(value) => (
            &value.name,
            SymbolKind::MutableVariable,
            value.span,
            value.visibility,
            named_type(&value.type_expr),
            format!(
                "mutable var {}: {}",
                value.name,
                type_text(snapshot, &value.type_expr)
            ),
        ),
        Decl::TypeDef(value) => (
            &value.name,
            if matches!(value.body, TypeBody::Enum(_)) {
                SymbolKind::Enum
            } else {
                SymbolKind::Type
            },
            value.span,
            value.visibility,
            match &value.body {
                TypeBody::Alias(target) => named_type(target),
                TypeBody::Record(_) | TypeBody::Enum(_) => Some(value.name.clone()),
            },
            format!("type {}", value.name),
        ),
        Decl::Function(value) => (
            &value.name,
            SymbolKind::Function,
            value.span,
            value.visibility,
            named_type(&value.return_type),
            function_detail(snapshot, value),
        ),
        Decl::Procedure(value) => (
            &value.name,
            SymbolKind::Procedure,
            value.span,
            value.visibility,
            None,
            procedure_detail(snapshot, value),
        ),
    };
    let full_span = span.into();
    let qualified_name = format!("{owner}.{name}");
    let callable = match declaration {
        Decl::Const(value) => type_callable_signature(snapshot, name, &value.type_expr),
        Decl::Var(value) | Decl::MutableVar(value) => {
            type_callable_signature(snapshot, name, &value.type_expr)
        }
        Decl::Function(value) => Some(function_signature(snapshot, value, 0)),
        Decl::Procedure(value) => Some(procedure_signature(snapshot, value, 0)),
        Decl::TypeDef(_) => None,
    };
    let children = match declaration {
        Decl::TypeDef(value) => members::type_children(
            snapshot,
            &qualified_name,
            &value.body,
            full_span,
            scope_span,
        ),
        Decl::Function(value) => function_children(snapshot, &qualified_name, value),
        Decl::Procedure(value) => procedure_children(snapshot, &qualified_name, value),
        _ => Vec::new(),
    };
    DocumentSymbol {
        name: name.clone(),
        qualified_name,
        kind,
        selection_span: name_span(snapshot, full_span, name),
        full_span,
        scope_span,
        visible_from: span.offset,
        visibility: visibility(decl_visibility),
        type_name,
        detail,
        callable,
        children,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn member_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    name: &str,
    kind: SymbolKind,
    span: Span,
    visibility_value: Visibility,
    type_name: Option<String>,
    detail: String,
    scope_span: SourceSpan,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol {
    let full_span = span.into();
    DocumentSymbol {
        name: name.to_owned(),
        qualified_name: format!("{owner}.{name}"),
        kind,
        full_span,
        selection_span: name_span(snapshot, full_span, name),
        scope_span,
        visible_from: span.offset,
        visibility: visibility(visibility_value),
        type_name,
        detail,
        callable: None,
        children,
    }
}

fn visibility(value: Visibility) -> SymbolVisibility {
    match value {
        Visibility::Public => SymbolVisibility::Public,
        Visibility::Private => SymbolVisibility::Private,
    }
}
