//! Local declarations and decision scopes in statement bodies.

use fpas_diagnostics::SourceSpan;
use fpas_parser::{Stmt, Visibility};

use super::expressions::{collect_designator_symbols, collect_expression_symbols};
use super::patterns::collect_case_bindings;
use super::{member_symbol, named_type, type_callable_signature, type_text};
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind};

/// Collects local symbols and recursively visits statement value positions.
pub(super) fn collect_statement_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    statements: &[Stmt],
    scope_span: SourceSpan,
    output: &mut Vec<DocumentSymbol>,
) {
    for statement in statements {
        match statement {
            Stmt::Const(value) | Stmt::Var(value) => {
                let constant = matches!(statement, Stmt::Const(_));
                let keyword = if constant { "const" } else { "var" };
                let annotation = value
                    .type_expr
                    .as_ref()
                    .map_or_else(String::new, |ty| format!(": {}", type_text(snapshot, ty)));
                let mut symbol = member_symbol(
                    snapshot,
                    owner,
                    &value.name,
                    if constant {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    },
                    value.span,
                    Visibility::Private,
                    value.type_expr.as_ref().and_then(named_type),
                    format!("{keyword} {}{annotation}", value.name),
                    scope_span,
                    Vec::new(),
                );
                symbol.callable = value
                    .type_expr
                    .as_ref()
                    .and_then(|ty| type_callable_signature(snapshot, &value.name, ty));
                symbol.children = value.type_expr.as_ref().map_or_else(Vec::new, |ty| {
                    super::source::callable_type_children(snapshot, owner, ty)
                });
                output.push(symbol);
                collect_expression_symbols(snapshot, owner, &value.value, output);
            }
            Stmt::Block(body, span) | Stmt::Scope { body, span } => {
                collect_statement_symbols(
                    snapshot,
                    owner,
                    body,
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                collect_expression_symbols(snapshot, owner, condition, output);
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
                match statement {
                    Stmt::For { start, end, .. } => {
                        collect_expression_symbols(snapshot, owner, start, output);
                        collect_expression_symbols(snapshot, owner, end, output);
                    }
                    Stmt::ForIn { iterable, .. } => {
                        collect_expression_symbols(snapshot, owner, iterable, output);
                    }
                    _ => unreachable!(),
                }
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
            Stmt::While {
                condition,
                body,
                span,
            } => {
                collect_expression_symbols(snapshot, owner, condition, output);
                collect_statement_symbols(
                    snapshot,
                    owner,
                    std::slice::from_ref(body.as_ref()),
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
            }
            Stmt::Repeat {
                condition,
                body,
                span,
            } => {
                collect_statement_symbols(
                    snapshot,
                    owner,
                    body,
                    span.diagnostic_span_or_synthetic(),
                    output,
                );
                collect_expression_symbols(snapshot, owner, condition, output);
            }
            Stmt::Case {
                expr,
                arms,
                else_body,
                span,
            } => {
                collect_expression_symbols(snapshot, owner, expr, output);
                for arm in arms {
                    let arm_scope = arm.span.diagnostic_span_or_synthetic();
                    collect_case_bindings(snapshot, owner, &arm.labels, arm_scope, output);
                    if let Some(guard) = &arm.guard {
                        collect_expression_symbols(snapshot, owner, guard, output);
                    }
                    collect_statement_symbols(
                        snapshot,
                        owner,
                        std::slice::from_ref(&arm.body),
                        arm_scope,
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
            Stmt::Assign { target, value, .. } => {
                collect_designator_symbols(snapshot, owner, target, output);
                collect_expression_symbols(snapshot, owner, value, output);
            }
            Stmt::Call {
                designator, args, ..
            } => {
                collect_designator_symbols(snapshot, owner, designator, output);
                for argument in args {
                    collect_expression_symbols(snapshot, owner, argument, output);
                }
            }
            Stmt::Return(Some(value), _)
            | Stmt::Panic(value, _)
            | Stmt::Expression { expr: value, .. }
            | Stmt::Discard { expr: value, .. } => {
                collect_expression_symbols(snapshot, owner, value, output);
            }
            Stmt::Return(None, _) | Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }
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
        branch.span().diagnostic_span_or_synthetic(),
        output,
    );
}
