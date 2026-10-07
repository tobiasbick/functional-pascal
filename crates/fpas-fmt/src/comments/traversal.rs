//! Recursive AST traversal for leading, trailing, and callable-body comment anchors.

mod declarations;
mod expressions;
use declarations::{collect_closer, collect_decls};

use std::collections::{BTreeMap, BTreeSet};

use fpas_lexer::{Span, Token, lex_with_comments};
use fpas_parser::{CaseArm, CaseLabel, CompilationUnit, Decl, Program, Stmt, Unit};

use super::anchors::{EmissionAnchor, span_end, stmt_end, stmt_start};
use expressions::{collect_designator, collect_expr};

/// Complete set of source positions consumed by comment emission.
#[derive(Debug, Default)]
pub(crate) struct CollectedAnchors {
    pub leading: Vec<usize>,
    pub emission: Vec<EmissionAnchor>,
    pub bodies: BTreeMap<usize, usize>,
    pub headers: BTreeMap<usize, usize>,
    pub declarations: BTreeSet<usize>,
    pub closers: BTreeMap<usize, usize>,
    semicolons: Vec<EmissionAnchor>,
    pub named_ends: Vec<EmissionAnchor>,
    else_keywords: Vec<usize>,
}

/// Collects every AST and keyword anchor required to preserve source comments.
#[must_use]
pub(crate) fn collect(unit: &CompilationUnit, source: &str) -> CollectedAnchors {
    let (tokens, _, _) = lex_with_comments(source);
    let begin_offsets: Vec<usize> = tokens
        .iter()
        .filter(|token| token.token == Token::Begin)
        .map(|token| token.span.offset)
        .collect();
    let mut anchors = CollectedAnchors {
        named_ends: tokens
            .windows(2)
            .filter_map(|pair| {
                (pair[0].token == Token::End
                    && matches!(
                        pair[1].token,
                        Token::Function
                            | Token::Procedure
                            | Token::Record
                            | Token::Enum
                            | Token::Unit
                            | Token::If
                            | Token::Case
                            | Token::For
                            | Token::While
                            | Token::With
                    ))
                .then_some(EmissionAnchor {
                    start: pair[0].span.offset,
                    end: span_end(pair[1].span),
                })
            })
            .collect(),
        else_keywords: tokens
            .iter()
            .filter(|token| token.token == Token::Else)
            .map(|token| token.span.offset)
            .collect(),
        semicolons: tokens
            .iter()
            .filter(|token| token.token == Token::Semicolon)
            .map(|token| EmissionAnchor {
                start: token.span.offset,
                end: span_end(token.span),
            })
            .collect(),
        ..CollectedAnchors::default()
    };
    anchors.leading.extend(tokens.iter().filter_map(|token| {
        matches!(token.token, Token::Uses | Token::Begin).then_some(token.span.offset)
    }));

    match unit {
        CompilationUnit::Program(program) => {
            collect_program(program, source, &begin_offsets, &mut anchors)
        }
        CompilationUnit::Unit(unit) => collect_unit(unit, source, &begin_offsets, &mut anchors),
    }
    anchors.leading.sort_unstable();
    anchors.leading.dedup();
    anchors
}

fn collect_program(program: &Program, source: &str, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(program.span.offset);
    out.declarations.insert(program.span.offset);
    push_span(program.span, out);
    for name in &program.uses {
        out.leading.push(name.span.offset);
        push_uses_span(name.span, source, out);
    }
    collect_decls(&program.declarations, begins, out);
    collect_stmts(&program.body, begins, out);
    collect_body(
        program.span.offset,
        program.span,
        &program.declarations,
        &program.body,
        begins,
        out,
    );
    let header_boundary = program
        .uses
        .first()
        .map(|name| name.span.offset)
        .or_else(|| program.declarations.first().map(crate::span::decl_span))
        .or_else(|| out.bodies.get(&program.span.offset).copied())
        .unwrap_or_else(|| span_end(program.span));
    collect_header(program.span.offset, header_boundary, out);
}

fn collect_unit(unit: &Unit, source: &str, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(unit.span.offset);
    out.declarations.insert(unit.span.offset);
    push_span(unit.span, out);
    collect_closer(unit.span, out);
    for name in &unit.uses {
        out.leading.push(name.span.offset);
        push_uses_span(name.span, source, out);
    }
    collect_decls(&unit.declarations, begins, out);
    let header_boundary = unit
        .uses
        .first()
        .map(|name| name.span.offset)
        .or_else(|| unit.declarations.first().map(crate::span::decl_span))
        .or_else(|| out.closers.get(&unit.span.offset).copied())
        .unwrap_or_else(|| span_end(unit.span));
    collect_header(unit.span.offset, header_boundary, out);
}

fn collect_header(owner_start: usize, boundary: usize, out: &mut CollectedAnchors) {
    if let Some(anchor) = out
        .semicolons
        .iter()
        .copied()
        .rfind(|anchor| anchor.start >= owner_start && anchor.end <= boundary)
    {
        out.emission.push(anchor);
        out.headers.insert(owner_start, anchor.start);
    }
}

fn collect_body(
    owner_start: usize,
    owner_span: Span,
    nested: &[Decl],
    stmts: &[Stmt],
    begins: &[usize],
    out: &mut CollectedAnchors,
) {
    let lower = nested.last().map(decl_end).unwrap_or(owner_span.offset);
    let upper = stmts
        .first()
        .map(stmt_start)
        .unwrap_or_else(|| span_end(owner_span));
    if let Some(begin) = begins
        .iter()
        .copied()
        .rfind(|offset| *offset >= lower && *offset < upper)
    {
        out.bodies.insert(owner_start, begin);
    }
}

fn collect_stmts(stmts: &[Stmt], begins: &[usize], out: &mut CollectedAnchors) {
    for stmt in stmts {
        collect_nested_stmt(stmt, begins, out);
    }
}

fn collect_nested_stmt(stmt: &Stmt, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(stmt_start(stmt));
    if matches!(stmt, Stmt::Const(_) | Stmt::Var(_)) {
        out.declarations.insert(stmt_start(stmt));
    }
    out.emission.push(EmissionAnchor {
        start: stmt_start(stmt),
        end: stmt_end(stmt),
    });
    collect_stmt_contents(stmt, begins, out);
}

fn collect_branch_stmt(stmt: &Stmt, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(stmt_start(stmt));
    if !matches!(stmt, Stmt::Block(..)) {
        out.emission.push(EmissionAnchor {
            start: stmt_start(stmt),
            end: stmt_end(stmt),
        });
    }
    collect_stmt_contents(stmt, begins, out);
}

fn collect_stmt_contents(stmt: &Stmt, begins: &[usize], out: &mut CollectedAnchors) {
    match stmt {
        Stmt::Block(stmts, _) => collect_stmts(stmts, begins, out),
        Stmt::Const(var) | Stmt::Var(var) => collect_expr(&var.value, begins, out),
        Stmt::Assign { target, value, .. } => {
            collect_designator(target, begins, out);
            collect_expr(value, begins, out);
        }
        Stmt::Return(value, _) => {
            if let Some(value) = value {
                collect_expr(value, begins, out);
            }
        }
        Stmt::Panic(value, _) => collect_expr(value, begins, out),
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span,
        } => {
            collect_expr(condition, begins, out);
            collect_closer(*span, out);
            collect_branch_stmt(then_branch, begins, out);
            if let Some(branch) = else_branch {
                collect_branch_stmt(branch, begins, out);
            }
        }
        Stmt::Case {
            expr,
            arms,
            else_body,
            span,
        } => {
            collect_closer(*span, out);
            collect_expr(expr, begins, out);
            for arm in arms {
                collect_case_arm(arm, begins, out);
            }
            if let Some(stmts) = else_body {
                let lower = arms.last().map_or(span.offset, |arm| span_end(arm.span));
                let upper = stmts.first().map_or(span_end(*span), stmt_start);
                if let Some(anchor) = out
                    .else_keywords
                    .iter()
                    .copied()
                    .find(|offset| *offset >= lower && *offset < upper)
                {
                    out.bodies.insert(span.offset, anchor);
                    out.leading.push(anchor);
                }
                collect_stmts(stmts, begins, out);
            }
        }
        Stmt::For {
            start,
            end,
            body,
            span,
            ..
        } => {
            collect_closer(*span, out);
            collect_expr(start, begins, out);
            collect_expr(end, begins, out);
            collect_branch_stmt(body, begins, out);
        }
        Stmt::ForIn {
            iterable,
            body,
            span,
            ..
        } => {
            collect_closer(*span, out);
            collect_expr(iterable, begins, out);
            collect_branch_stmt(body, begins, out);
        }
        Stmt::While {
            condition,
            body,
            span,
        } => {
            collect_closer(*span, out);
            collect_expr(condition, begins, out);
            collect_branch_stmt(body, begins, out);
        }
        Stmt::Repeat {
            body, condition, ..
        } => {
            collect_stmts(body, begins, out);
            collect_expr(condition, begins, out);
        }
        Stmt::Call {
            designator, args, ..
        } => {
            collect_designator(designator, begins, out);
            for arg in args {
                collect_expr(arg, begins, out);
            }
        }
        Stmt::Expression { expr, .. } | Stmt::Go { expr, .. } | Stmt::Discard { expr, .. } => {
            collect_expr(expr, begins, out)
        }
        Stmt::Null(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn collect_case_arm(arm: &CaseArm, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(arm.span.offset);
    push_span(arm.span, out);
    for label in &arm.labels {
        if let CaseLabel::Value { start, end, .. } = label {
            collect_expr(start, begins, out);
            if let Some(end) = end {
                collect_expr(end, begins, out);
            }
        }
    }
    if let Some(guard) = &arm.guard {
        collect_expr(guard, begins, out);
    }
    collect_branch_stmt(&arm.body, begins, out);
}

fn push_span(span: Span, out: &mut CollectedAnchors) {
    out.emission.push(EmissionAnchor {
        start: span.offset,
        end: span_end(span),
    });
}

fn push_uses_span(span: Span, source: &str, out: &mut CollectedAnchors) {
    let name_end = span_end(span);
    let delimiter_end = source.get(name_end..).and_then(|suffix| {
        suffix
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .and_then(|(offset, ch)| (ch == ',').then_some(name_end + offset + ch.len_utf8()))
    });
    out.emission.push(EmissionAnchor {
        start: span.offset,
        end: delimiter_end.unwrap_or(name_end),
    });
}

fn decl_end(decl: &Decl) -> usize {
    match decl {
        Decl::Const(def) => span_end(def.span),
        Decl::Var(def) => span_end(def.span),
        Decl::TypeDef(def) => span_end(def.span),
        Decl::Function(function) => span_end(function.span),
        Decl::Procedure(procedure) => span_end(procedure.span),
    }
}
