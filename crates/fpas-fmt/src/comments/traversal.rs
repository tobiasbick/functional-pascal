//! Recursive AST traversal for leading, trailing, and callable-body comment anchors.

mod expressions;

use std::collections::{BTreeMap, BTreeSet};

use fpas_lexer::{Span, Token, lex_with_comments};
use fpas_parser::{
    CaseArm, CaseLabel, CompilationUnit, Decl, FuncBody, Program, RecordMethod, Stmt, TypeBody,
    Unit,
};

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
    semicolons: Vec<EmissionAnchor>,
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
        CompilationUnit::Program(program) => collect_program(program, &begin_offsets, &mut anchors),
        CompilationUnit::Unit(unit) => collect_unit(unit, &begin_offsets, &mut anchors),
    }
    anchors.leading.sort_unstable();
    anchors.leading.dedup();
    anchors
}

fn collect_program(program: &Program, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(program.span.offset);
    out.declarations.insert(program.span.offset);
    push_span(program.span, out);
    out.leading
        .extend(program.uses.iter().map(|name| name.span.offset));
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

fn collect_unit(unit: &Unit, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(unit.span.offset);
    out.declarations.insert(unit.span.offset);
    out.leading
        .extend(unit.uses.iter().map(|name| name.span.offset));
    collect_decls(&unit.declarations, begins, out);
    let header_boundary = unit
        .uses
        .first()
        .map(|name| name.span.offset)
        .or_else(|| unit.declarations.first().map(crate::span::decl_span))
        .unwrap_or_else(|| span_end(unit.span));
    collect_header(unit.span.offset, header_boundary, out);
}

fn collect_decls(decls: &[Decl], begins: &[usize], out: &mut CollectedAnchors) {
    for decl in decls {
        let start = crate::span::decl_span(decl);
        out.leading.push(start);
        out.declarations.insert(start);
        match decl {
            Decl::Const(def) => {
                push_span(def.span, out);
                collect_expr(&def.value, begins, out);
            }
            Decl::Var(def) | Decl::MutableVar(def) => {
                push_span(def.span, out);
                collect_expr(&def.value, begins, out);
            }
            Decl::TypeDef(def) => {
                push_span(def.span, out);
                match &def.body {
                    TypeBody::Record(record) => {
                        for field in &record.fields {
                            out.leading.push(field.span.offset);
                            out.declarations.insert(field.span.offset);
                            push_span(field.span, out);
                            if let Some(value) = &field.default_value {
                                collect_expr(value, begins, out);
                            }
                        }
                        for method in &record.methods {
                            match method {
                                RecordMethod::Function(function)
                                | RecordMethod::StaticFunction(function) => {
                                    collect_routine(function.span, &function.body, begins, out);
                                }
                                RecordMethod::Procedure(procedure)
                                | RecordMethod::StaticProcedure(procedure) => {
                                    collect_routine(procedure.span, &procedure.body, begins, out);
                                }
                            }
                        }
                        for property in &record.properties {
                            out.leading.push(property.span.offset);
                            out.declarations.insert(property.span.offset);
                            push_span(property.span, out);
                        }
                        for event in &record.events {
                            out.leading.push(event.span.offset);
                            out.declarations.insert(event.span.offset);
                            push_span(event.span, out);
                        }
                    }
                    TypeBody::Enum(enum_type) => {
                        for member in &enum_type.members {
                            out.leading.push(member.span.offset);
                            out.declarations.insert(member.span.offset);
                            push_span(member.span, out);
                        }
                    }
                    TypeBody::Alias(_) => {}
                }
            }
            Decl::Function(function) => collect_routine(function.span, &function.body, begins, out),
            Decl::Procedure(procedure) => {
                collect_routine(procedure.span, &procedure.body, begins, out)
            }
        }
    }
}

fn collect_routine(span: Span, body: &FuncBody, begins: &[usize], out: &mut CollectedAnchors) {
    out.leading.push(span.offset);
    out.declarations.insert(span.offset);
    push_span(span, out);
    let FuncBody::Block { nested, stmts } = body;
    collect_decls(nested, begins, out);
    collect_stmts(stmts, begins, out);
    collect_body(span.offset, span, nested, stmts, begins, out);
    let header_boundary = nested
        .first()
        .map(crate::span::decl_span)
        .or_else(|| out.bodies.get(&span.offset).copied())
        .unwrap_or_else(|| span_end(span));
    collect_header(span.offset, header_boundary, out);
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
    if matches!(stmt, Stmt::Var(_) | Stmt::MutableVar(_)) {
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
        Stmt::Var(var) | Stmt::MutableVar(var) => collect_expr(&var.value, begins, out),
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
            ..
        } => {
            collect_expr(condition, begins, out);
            collect_branch_stmt(then_branch, begins, out);
            if let Some(branch) = else_branch {
                collect_branch_stmt(branch, begins, out);
            }
        }
        Stmt::Case {
            expr,
            arms,
            else_body,
            ..
        } => {
            collect_expr(expr, begins, out);
            for arm in arms {
                collect_case_arm(arm, begins, out);
            }
            if let Some(stmts) = else_body {
                if let [stmt] = stmts.as_slice() {
                    collect_branch_stmt(stmt, begins, out);
                } else {
                    collect_stmts(stmts, begins, out);
                }
            }
        }
        Stmt::For {
            start, end, body, ..
        } => {
            collect_expr(start, begins, out);
            collect_expr(end, begins, out);
            collect_branch_stmt(body, begins, out);
        }
        Stmt::ForIn { iterable, body, .. } => {
            collect_expr(iterable, begins, out);
            collect_branch_stmt(body, begins, out);
        }
        Stmt::While {
            condition, body, ..
        } => {
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
        Stmt::Expression { expr, .. } | Stmt::Go { expr, .. } => collect_expr(expr, begins, out),
        Stmt::Break(_) | Stmt::Continue(_) => {}
    }
}

fn collect_case_arm(arm: &CaseArm, begins: &[usize], out: &mut CollectedAnchors) {
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

fn decl_end(decl: &Decl) -> usize {
    match decl {
        Decl::Const(def) => span_end(def.span),
        Decl::Var(def) | Decl::MutableVar(def) => span_end(def.span),
        Decl::TypeDef(def) => span_end(def.span),
        Decl::Function(function) => span_end(function.span),
        Decl::Procedure(procedure) => span_end(procedure.span),
    }
}
