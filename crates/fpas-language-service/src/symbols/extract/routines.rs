//! Routine parameters, nested declarations and body scopes.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Span;
use fpas_parser::{
    ClosureExpr, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, TypeParam, Visibility,
};

use super::statements::collect_statement_symbols;
use super::type_parameters::type_parameter_symbols;
use super::{declaration_symbol, member_symbol, named_type, type_callable_signature, type_text};
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind};

/// Extracts function parameters, body declarations and postcondition result names.
pub(crate) fn function_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &FunctionDecl,
) -> Vec<DocumentSymbol> {
    let mut children = routine_children(
        snapshot,
        owner,
        &declaration.type_params,
        &declaration.params,
        &declaration.body,
        declaration.span,
    );
    super::contracts::collect_result_symbols(snapshot, owner, declaration, &mut children);
    children.extend(super::source::callable_type_children(
        snapshot,
        owner,
        &declaration.return_type,
    ));
    children.sort_by_key(|symbol| symbol.full_span.offset());
    children
}

/// Extracts procedure parameters and scoped body declarations.
pub(crate) fn procedure_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &ProcedureDecl,
) -> Vec<DocumentSymbol> {
    routine_children(
        snapshot,
        owner,
        &declaration.type_params,
        &declaration.params,
        &declaration.body,
        declaration.span,
    )
}

/// Extracts closure parameters and declarations within the closure's source scope.
pub(super) fn closure_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    closure: &ClosureExpr,
) -> Vec<DocumentSymbol> {
    let mut children = routine_children(
        snapshot,
        owner,
        &[],
        &closure.params,
        &closure.body,
        closure.span,
    );
    if let Some(return_type) = &closure.return_type {
        children.extend(super::source::callable_type_children(
            snapshot,
            owner,
            return_type,
        ));
    }
    children
}

fn routine_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    type_params: &[TypeParam],
    params: &[FormalParam],
    body: &FuncBody,
    routine_span: Span,
) -> Vec<DocumentSymbol> {
    let scope_span = routine_span.diagnostic_span_or_synthetic();
    let mut children = type_parameter_symbols(snapshot, owner, type_params, scope_span)
        .into_iter()
        .chain(
            params
                .iter()
                .map(|param| parameter_symbol(snapshot, owner, param, scope_span)),
        )
        .collect::<Vec<_>>();
    let FuncBody::Block { nested, stmts } = body;
    children.extend(
        nested
            .iter()
            .map(|declaration| declaration_symbol(snapshot, owner, declaration, scope_span)),
    );
    collect_statement_symbols(snapshot, owner, stmts, scope_span, &mut children);
    children.sort_by_key(|symbol| symbol.full_span.offset());
    children
}

/// Builds a source-owned parameter, including its callable type labels.
pub(super) fn parameter_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    param: &FormalParam,
    scope_span: SourceSpan,
) -> DocumentSymbol {
    let mut symbol = member_symbol(
        snapshot,
        owner,
        &param.name,
        SymbolKind::Parameter,
        param.span,
        Visibility::Private,
        named_type(&param.type_expr),
        format!(
            "{} {}: {}",
            if param.mode == fpas_parser::ParamMode::Var {
                "var parameter"
            } else {
                "parameter"
            },
            param.name,
            type_text(snapshot, &param.type_expr)
        ),
        scope_span,
        Vec::new(),
    );
    symbol.callable = type_callable_signature(snapshot, &param.name, &param.type_expr);
    symbol.children = super::source::callable_type_children(snapshot, owner, &param.type_expr);
    symbol
}
