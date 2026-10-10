//! Recursive syntax-to-type mapping for recovered native receiver chains.
//! Reference: `docs/pascal/tools/editor-integration.md`.

use super::MAX_RECEIVER_DEPTH;
use crate::navigation::{NavigationDocument, find_type};
use crate::{DocumentSymbol, SymbolKind};
use fpas_parser::{CompilationUnit, Decl, FormalParam, TypeBody, TypeExpr};
use fpas_sema::{FunctionTy, ParamTy, ProcedureTy, Ty};

/// Resolves a declaration's value type, retaining the signature of a routine value.
/// See `docs/pascal/tools/editor-integration.md`.
pub(super) fn from_symbol(
    documents: &[NavigationDocument],
    owner: usize,
    symbol: &DocumentSymbol,
    depth: usize,
) -> Option<Ty> {
    if matches!(
        symbol.kind,
        SymbolKind::Function | SymbolKind::Procedure | SymbolKind::Method
    ) {
        let signature = &symbol.callable.as_ref()?.label;
        let kind = signature.split_whitespace().next()?;
        let parameters = signature.get(signature.find('(')?..)?;
        return from_text(documents, owner, &format!("{kind}{parameters}"), depth);
    }
    let name = symbol
        .type_name
        .as_deref()
        .or_else(|| symbol.detail.split_once(": ").map(|(_, ty)| ty))?;
    if let Some(semantic) = documents[owner]
        .analysis
        .as_ref()
        .and_then(|analysis| analysis.semantic())
        && let Some((_, ty)) = semantic
            .metadata()
            .named_types
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
    {
        return Some(ty.clone());
    }
    from_text(documents, owner, name, depth)
}

/// Parses a type fragment with the shared parser before resolving its receiver shape.
pub(super) fn from_text(
    documents: &[NavigationDocument],
    owner: usize,
    text: &str,
    depth: usize,
) -> Option<Ty> {
    let (syntax, diagnostics) = fpas_parser::parse_type_expression(text);
    if !diagnostics.is_empty() {
        return None;
    }
    from_syntax(documents, owner, &syntax, depth)
}

/// Maps recursive type syntax, resolving aliases in their declaring unit's import environment.
pub(super) fn from_syntax(
    documents: &[NavigationDocument],
    owner: usize,
    syntax: &TypeExpr,
    depth: usize,
) -> Option<Ty> {
    if depth > MAX_RECEIVER_DEPTH {
        return None;
    }
    let resolve = |inner| from_syntax(documents, owner, inner, depth + 1).map(Box::new);
    Some(match syntax {
        TypeExpr::Named { id, .. } => {
            let name = id.parts.join(".");
            match name.to_ascii_lowercase().as_str() {
                "string" => Ty::String,
                "integer" => Ty::Integer,
                "boolean" => Ty::Boolean,
                "real" => Ty::Real,
                "task" => Ty::Task(Box::new(Ty::Error)),
                _ => return named_type(documents, owner, &name, depth),
            }
        }
        TypeExpr::Array(inner, _) => Ty::Array(resolve(inner)?),
        TypeExpr::Channel(inner, _) => Ty::Channel(resolve(inner)?),
        TypeExpr::Task(inner, _) => Ty::Task(resolve(inner)?),
        TypeExpr::Option { inner_type, .. } => Ty::Option(resolve(inner_type)?),
        TypeExpr::Result {
            ok_type, err_type, ..
        } => Ty::Result(resolve(ok_type)?, resolve(err_type)?),
        TypeExpr::Dict {
            key_type,
            value_type,
            ..
        } => Ty::Dict(resolve(key_type)?, resolve(value_type)?),
        TypeExpr::FunctionType {
            params,
            return_type,
            ..
        } => Ty::Function(FunctionTy {
            type_params: Vec::new(),
            params: parameters(documents, owner, params, depth)?,
            return_type: resolve(return_type)?,
            variadic: false,
        }),
        TypeExpr::ProcedureType { params, .. } => Ty::Procedure(ProcedureTy {
            type_params: Vec::new(),
            params: parameters(documents, owner, params, depth)?,
            variadic: false,
        }),
    })
}

fn parameters(
    documents: &[NavigationDocument],
    owner: usize,
    params: &[FormalParam],
    depth: usize,
) -> Option<Vec<ParamTy>> {
    params
        .iter()
        .map(|param| {
            Some(ParamTy {
                name: param.name.clone(),
                mode: param.mode,
                ty: from_syntax(documents, owner, &param.type_expr, depth + 1)?,
            })
        })
        .collect()
}

fn named_type(
    documents: &[NavigationDocument],
    owner: usize,
    name: &str,
    depth: usize,
) -> Option<Ty> {
    let Some((index, symbol)) = find_type(documents, owner, owner, name) else {
        return Some(Ty::Named(name.to_owned()));
    };
    let declarations = match documents[index].snapshot.compilation_unit() {
        CompilationUnit::Program(program) => &program.declarations,
        CompilationUnit::Unit(unit) => &unit.declarations,
    };
    let definition = declarations
        .iter()
        .find_map(|declaration| match declaration {
            Decl::TypeDef(definition) if definition.span.offset == symbol.full_span.offset() => {
                Some(definition)
            }
            _ => None,
        })?;
    match &definition.body {
        TypeBody::Alias(syntax) => from_syntax(documents, index, syntax, depth + 1),
        _ => Some(Ty::Named(symbol.qualified_name.clone())),
    }
}
