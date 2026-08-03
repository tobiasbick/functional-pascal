//! Routine parameters, nested declarations, and statement-local symbols.

use fpas_diagnostics::SourceSpan;
use fpas_lexer::Span;
use fpas_parser::{
    Decl, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, Stmt, TypeParam, Visibility,
};

use super::{
    declaration_symbol, member_symbol, name_span, named_type, type_callable_signature, type_text,
};
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind, SymbolVisibility};

pub(crate) fn function_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &FunctionDecl,
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

pub(super) fn collect_statement_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    statements: &[Stmt],
    scope_span: SourceSpan,
    output: &mut Vec<DocumentSymbol>,
) {
    for statement in statements {
        match statement {
            Stmt::Var(value) | Stmt::MutableVar(value) => {
                let declaration = if matches!(statement, Stmt::Var(_)) {
                    Decl::Var(value.clone())
                } else {
                    Decl::MutableVar(value.clone())
                };
                output.push(declaration_symbol(
                    snapshot,
                    owner,
                    &declaration,
                    scope_span,
                ));
            }
            Stmt::Block(body, span) => {
                collect_statement_symbols(
                    snapshot,
                    owner,
                    body,
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_branch(snapshot, owner, then_branch, output);
                if let Some(branch) = else_branch {
                    collect_branch(snapshot, owner, branch, output);
                }
            }
            Stmt::For {
                var_name,
                var_type,
                body,
                span,
                ..
            }
            | Stmt::ForIn {
                var_name,
                var_type,
                body,
                span,
                ..
            } => {
                let mut symbol = member_symbol(
                    snapshot,
                    owner,
                    var_name,
                    SymbolKind::LoopVariable,
                    *span,
                    Visibility::Private,
                    named_type(var_type),
                    format!(
                        "loop variable {var_name}: {}",
                        type_text(snapshot, var_type)
                    ),
                    span.diagnostic_span_or_synthetic(),
                    Vec::new(),
                );
                symbol.callable = type_callable_signature(snapshot, var_name, var_type);
                output.push(symbol);
                collect_statement_symbols(
                    snapshot,
                    owner,
                    std::slice::from_ref(body.as_ref()),
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
            }
            Stmt::While { body, span, .. } => collect_statement_symbols(
                snapshot,
                owner,
                std::slice::from_ref(body.as_ref()),
                span.diagnostic_span_or_synthetic(),
                output,
            ),
            Stmt::Repeat { body, span, .. } => {
                collect_statement_symbols(
                    snapshot,
                    owner,
                    body,
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
            }
            Stmt::Case {
                arms,
                else_body,
                span,
                ..
            } => {
                for arm in arms {
                    collect_statement_symbols(
                        snapshot,
                        owner,
                        std::slice::from_ref(&arm.body),
                        arm.span.diagnostic_span_or_synthetic(),
                        output,
                    );
                }
                if let Some(body) = else_body {
                    collect_statement_symbols(
                        snapshot,
                        owner,
                        body,
                        span.diagnostic_span_or_synthetic(),
                        output,
                    );
                }
            }
            _ => {}
        }
    }
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
    let mut children = type_params
        .iter()
        .map(|parameter| type_parameter_symbol(snapshot, owner, parameter, scope_span))
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

fn type_parameter_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    parameter: &TypeParam,
    scope_span: SourceSpan,
) -> DocumentSymbol {
    let selection_span = name_span(snapshot, scope_span, &parameter.name);
    let constraint = parameter
        .constraint
        .as_deref()
        .map_or_else(String::new, |value| format!(": {value}"));
    DocumentSymbol {
        name: parameter.name.clone(),
        qualified_name: format!("{owner}.{}", parameter.name),
        kind: SymbolKind::TypeParameter,
        full_span: selection_span,
        selection_span,
        scope_span,
        visible_from: scope_span.offset(),
        visibility: SymbolVisibility::Private,
        type_name: None,
        detail: format!("type parameter {}{constraint}", parameter.name),
        callable: None,
        children: Vec::new(),
    }
}

fn parameter_symbol(
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
            "{}parameter {}: {}",
            if param.mutable { "mutable " } else { "" },
            param.name,
            type_text(snapshot, &param.type_expr)
        ),
        scope_span,
        Vec::new(),
    );
    symbol.callable = type_callable_signature(snapshot, &param.name, &param.type_expr);
    symbol
}

fn collect_branch(
    snapshot: &DocumentSnapshot,
    owner: &str,
    branch: &Stmt,
    output: &mut Vec<DocumentSymbol>,
) {
    collect_statement_symbols(
        snapshot,
        owner,
        std::slice::from_ref(branch),
        stmt_span(branch).diagnostic_span_or_synthetic(),
        output,
    );
}

fn stmt_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Block(_, span)
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::Break(span)
        | Stmt::Continue(span) => *span,
        Stmt::Var(value) | Stmt::MutableVar(value) => value.span,
        Stmt::Assign { span, .. }
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => *span,
    }
}
