//! Extract expressions only where the new binding preserves evaluation order and frequency.

use std::collections::HashSet;
use std::ops::Range;

use fpas_diagnostics::SourceSpan;
use fpas_lexer::{Span, Token, lex};
use fpas_parser::{
    CompilationUnit, Decl, Designator, DesignatorPart, Expr, FuncBody, RecordMethod, Stmt, TypeBody,
};

use crate::navigation::{NavigationDocument, resolve};
use crate::{SemanticEdit, SymbolKind};

struct StaticCallees<'a> {
    documents: &'a [NavigationDocument],
    target: usize,
}

pub(super) fn extract_variable(
    documents: &[NavigationDocument],
    target: usize,
    mut selection: Range<usize>,
) -> Option<Vec<SemanticEdit>> {
    let snapshot = &documents[target].snapshot;
    let source = snapshot.source();
    let fragment = source.get(selection.clone())?;
    selection.start += fragment.len() - fragment.trim_start().len();
    selection.end = selection.start + fragment.trim().len();
    if selection.is_empty() {
        return None;
    }
    let (declarations, body) = match snapshot.compilation_unit() {
        CompilationUnit::Program(program) => (&program.declarations, program.body.as_slice()),
        CompilationUnit::Unit(unit) => (&unit.declarations, &[][..]),
    };
    let aliases = StaticCallees { documents, target };
    let statement = find_statements(body, &selection, &aliases)
        .or_else(|| find_declarations(declarations, &selection, &aliases))?;
    let line_start = source[..statement.offset]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let prefix = source.get(line_start..statement.offset)?;
    // LocalBinding spans start at the name, after the declaration keyword.
    if !prefix.trim().is_empty()
        && !prefix.trim().eq_ignore_ascii_case("const")
        && !prefix.trim().eq_ignore_ascii_case("var")
    {
        return None;
    }
    let indentation = &prefix[..prefix.len() - prefix.trim_start().len()];
    let reserved = lex(source)
        .0
        .into_iter()
        .filter_map(|token| {
            if let Token::Ident(name) = token.token {
                Some(name.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    let mut name = "ExtractedValue".to_owned();
    let mut suffix = 2_u64;
    while reserved.contains(&name.to_ascii_lowercase()) {
        name = format!("ExtractedValue{suffix}");
        suffix = suffix.checked_add(1)?;
    }
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    Some(vec![
        SemanticEdit {
            span: SourceSpan::new(line_start, 0, 1, 1),
            new_text: format!(
                "{indentation}const {name} := {};{newline}",
                &source[selection.clone()]
            ),
        },
        SemanticEdit {
            span: SourceSpan::new(selection.start, selection.len(), 1, 1),
            new_text: name,
        },
    ])
}

fn find_declarations(
    declarations: &[Decl],
    selection: &Range<usize>,
    aliases: &StaticCallees<'_>,
) -> Option<Span> {
    declarations
        .iter()
        .find_map(|declaration| match declaration {
            Decl::Function(function) => find_body(&function.body, selection, aliases),
            Decl::Procedure(procedure) => find_body(&procedure.body, selection, aliases),
            Decl::TypeDef(definition) => match &definition.body {
                TypeBody::Record(record) => record.methods.iter().find_map(|method| match method {
                    RecordMethod::Function(function) | RecordMethod::StaticFunction(function) => {
                        find_body(&function.body, selection, aliases)
                    }
                    RecordMethod::Procedure(procedure)
                    | RecordMethod::StaticProcedure(procedure) => {
                        find_body(&procedure.body, selection, aliases)
                    }
                }),
                _ => None,
            },
            _ => None,
        })
}

fn find_body(
    body: &FuncBody,
    selection: &Range<usize>,
    aliases: &StaticCallees<'_>,
) -> Option<Span> {
    let FuncBody::Block { nested, stmts } = body;
    find_declarations(nested, selection, aliases)
        .or_else(|| find_statements(stmts, selection, aliases))
}

fn find_statements(
    statements: &[Stmt],
    selection: &Range<usize>,
    aliases: &StaticCallees<'_>,
) -> Option<Span> {
    statements
        .iter()
        .find_map(|statement| find_statement(statement, selection, aliases))
}

fn find_statement(
    statement: &Stmt,
    selection: &Range<usize>,
    aliases: &StaticCallees<'_>,
) -> Option<Span> {
    let span = statement.span();
    if selection.start < span.offset || selection.end > span.offset + span.length {
        return None;
    }
    let expression = match statement {
        Stmt::Const(binding) | Stmt::Var(binding) => Some(&binding.value),
        Stmt::Return(value, _) => value.as_ref(),
        Stmt::Panic(value, _) | Stmt::Discard { expr: value, .. } => Some(value),
        Stmt::Assign { target, value, .. } if target.parts.len() == 1 => Some(value),
        Stmt::Call {
            designator, args, ..
        } if static_callee(designator, aliases) => args.first().map(Expr::argument_value),
        _ => None,
    };
    if expression.is_some_and(|expression| first_evaluated(expression, selection, aliases)) {
        return Some(span);
    }
    match statement {
        Stmt::Block(body, _) | Stmt::Scope { body, .. } | Stmt::Repeat { body, .. } => {
            find_statements(body, selection, aliases)
        }
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => find_statement(then_branch, selection, aliases).or_else(|| {
            else_branch
                .as_ref()
                .and_then(|branch| find_statement(branch, selection, aliases))
        }),
        Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::ForIn { body, .. } => {
            find_statement(body, selection, aliases)
        }
        Stmt::Case {
            arms, else_body, ..
        } => arms
            .iter()
            .find_map(|arm| find_statement(&arm.body, selection, aliases))
            .or_else(|| {
                else_body
                    .as_ref()
                    .and_then(|body| find_statements(body, selection, aliases))
            }),
        _ => None,
    }
}

/// Moving a later operand, conditional branch, loop test, receiver or reference argument can
/// change effects, failure order, or iteration count. Follow only the unconditional first value.
fn first_evaluated(
    expression: &Expr,
    selection: &Range<usize>,
    aliases: &StaticCallees<'_>,
) -> bool {
    if matches!(
        expression,
        Expr::VarArgument { .. } | Expr::NamedArgument { .. } | Expr::Error(_)
    ) {
        return false;
    }
    let span = expression.span();
    if span.offset == selection.start && span.offset + span.length == selection.end {
        return true;
    }
    match expression {
        Expr::Paren(value, _)
        | Expr::UnaryOp { operand: value, .. }
        | Expr::ResultOk(value, _)
        | Expr::ResultError(value, _)
        | Expr::OptionSome(value, _)
        | Expr::Try(value, _)
        | Expr::Is { value, .. } => first_evaluated(value, selection, aliases),
        Expr::BinaryOp { left, .. } => first_evaluated(left, selection, aliases),
        Expr::Call {
            designator, args, ..
        } if static_callee(designator, aliases) => args
            .first()
            .is_some_and(|value| first_evaluated(value.argument_value(), selection, aliases)),
        Expr::Postfix { base, .. } | Expr::RecordUpdate { base, .. } => {
            first_evaluated(base, selection, aliases)
        }
        Expr::ArrayLiteral(values, _) => values
            .first()
            .is_some_and(|value| first_evaluated(value, selection, aliases)),
        Expr::DictLiteral(values, _) => values
            .first()
            .is_some_and(|(key, _)| first_evaluated(key, selection, aliases)),
        _ => false,
    }
}

fn static_callee(designator: &Designator, context: &StaticCallees<'_>) -> bool {
    if designator
        .parts
        .iter()
        .any(|part| !matches!(part, DesignatorPart::Ident(_, _)))
    {
        return false;
    }
    let Some(DesignatorPart::Ident(_, span)) = designator.parts.last() else {
        return false;
    };
    resolve(context.documents, context.target, span.offset).is_some_and(|(_, symbol, _)| {
        matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure)
    })
}
