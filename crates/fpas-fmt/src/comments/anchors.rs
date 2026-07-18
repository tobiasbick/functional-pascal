//! Source-span anchors for attaching comments during emission.

use fpas_lexer::{Span, Token, lex_with_comments};
use fpas_parser::{
    CaseArm, CompilationUnit, Decl, FuncBody, Program, RecordMethod, Stmt, TypeBody, Unit,
};

/// Start/end byte offsets of a formattable construct in source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmissionAnchor {
    pub start: usize,
    pub end: usize,
}

/// Returns the byte offset one past `span`.
#[must_use]
pub(crate) fn span_end(span: Span) -> usize {
    span.offset + span.length
}

/// Returns the lexer start offset for `stmt`.
#[must_use]
pub(crate) fn stmt_start(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Block(_, span) => span.offset,
        Stmt::Var(var) | Stmt::MutableVar(var) => var.span.offset,
        Stmt::Assign { span, .. }
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Break(span)
        | Stmt::Continue(span)
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => span.offset,
    }
}

/// Returns the lexer end offset for `stmt`.
#[must_use]
pub(crate) fn stmt_end(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::Block(_, span) => span_end(*span),
        Stmt::Var(var) | Stmt::MutableVar(var) => span_end(var.span),
        Stmt::Assign { span, .. }
        | Stmt::Return(_, span)
        | Stmt::Panic(_, span)
        | Stmt::If { span, .. }
        | Stmt::Case { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Repeat { span, .. }
        | Stmt::Break(span)
        | Stmt::Continue(span)
        | Stmt::Call { span, .. }
        | Stmt::Expression { span, .. }
        | Stmt::Go { span, .. } => span_end(*span),
    }
}

/// Sorted unique lexer offsets where leading comments may be emitted.
#[must_use]
pub(crate) fn collect_leading_anchor_offsets(unit: &CompilationUnit, source: &str) -> Vec<usize> {
    let mut offsets = Vec::new();
    match unit {
        CompilationUnit::Program(program) => push_program_offsets(program, &mut offsets),
        CompilationUnit::Unit(unit) => push_unit_offsets(unit, &mut offsets),
    }
    offsets.extend(keyword_leading_anchors(source));
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

/// All constructs that may carry same-line trailing comments.
#[must_use]
pub(crate) fn collect_emission_anchors(unit: &CompilationUnit) -> Vec<EmissionAnchor> {
    let mut anchors = Vec::new();
    match unit {
        CompilationUnit::Program(program) => {
            push_decl_anchors(&program.declarations, &mut anchors);
            push_stmts(&program.body, &mut anchors);
        }
        CompilationUnit::Unit(unit) => push_decl_anchors(&unit.declarations, &mut anchors),
    }
    anchors
}

/// Byte offset of the `uses` keyword when present.
#[must_use]
pub(crate) fn uses_keyword_offset(source: &str) -> Option<usize> {
    let (tokens, _, _) = lex_with_comments(source);
    tokens
        .iter()
        .find(|token| token.token == Token::Uses)
        .map(|token| token.span.offset)
}

/// Byte offset of the program body's `begin` keyword when present.
#[must_use]
pub(crate) fn begin_keyword_offset(source: &str) -> Option<usize> {
    let (tokens, _, _) = lex_with_comments(source);
    tokens
        .iter()
        .find(|token| token.token == Token::Begin)
        .map(|token| token.span.offset)
}

fn keyword_leading_anchors(source: &str) -> Vec<usize> {
    let (tokens, _, _) = lex_with_comments(source);
    tokens
        .iter()
        .filter_map(|token| match token.token {
            Token::Uses | Token::Begin => Some(token.span.offset),
            _ => None,
        })
        .collect()
}

fn push_program_offsets(program: &Program, out: &mut Vec<usize>) {
    out.push(program.span.offset);
    for unit_name in &program.uses {
        out.push(unit_name.span.offset);
    }
    push_decl_start_offsets(&program.declarations, out);
    push_stmt_start_offsets(&program.body, out);
}

fn push_unit_offsets(unit: &Unit, out: &mut Vec<usize>) {
    out.push(unit.span.offset);
    for unit_name in &unit.uses {
        out.push(unit_name.span.offset);
    }
    push_decl_start_offsets(&unit.declarations, out);
}

fn push_decl_start_offsets(decls: &[Decl], out: &mut Vec<usize>) {
    for decl in decls {
        out.push(crate::span::decl_span(decl));
        if let Decl::TypeDef(type_def) = decl
            && let TypeBody::Record(record) = &type_def.body
        {
            for method in &record.methods {
                match method {
                    RecordMethod::Function(function) | RecordMethod::StaticFunction(function) => {
                        out.push(function.span.offset)
                    }
                    RecordMethod::Procedure(procedure) => out.push(procedure.span.offset),
                }
            }
            for field in &record.fields {
                out.push(field.span.offset);
            }
            for property in &record.properties {
                out.push(property.span.offset);
            }
            for event in &record.events {
                out.push(event.span.offset);
            }
        }
    }
}

fn push_decl_anchors(decls: &[Decl], out: &mut Vec<EmissionAnchor>) {
    for decl in decls {
        match decl {
            Decl::Const(def) => push_var_like_anchor(def.span, out),
            Decl::Var(def) | Decl::MutableVar(def) => push_var_like_anchor(def.span, out),
            Decl::TypeDef(def) => {
                out.push(EmissionAnchor {
                    start: def.span.offset,
                    end: span_end(def.span),
                });
                if let TypeBody::Record(record) = &def.body {
                    for field in &record.fields {
                        out.push(EmissionAnchor {
                            start: field.span.offset,
                            end: span_end(field.span),
                        });
                    }
                    for method in &record.methods {
                        match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => {
                                push_routine_body_anchor(function.span, &function.body, out);
                            }
                            RecordMethod::Procedure(procedure) => {
                                push_routine_body_anchor(procedure.span, &procedure.body, out);
                            }
                        }
                    }
                    for property in &record.properties {
                        out.push(EmissionAnchor {
                            start: property.span.offset,
                            end: span_end(property.span),
                        });
                    }
                    for event in &record.events {
                        out.push(EmissionAnchor {
                            start: event.span.offset,
                            end: span_end(event.span),
                        });
                    }
                }
            }
            Decl::Function(function) => {
                push_routine_body_anchor(function.span, &function.body, out);
            }
            Decl::Procedure(procedure) => {
                push_routine_body_anchor(procedure.span, &procedure.body, out);
            }
        }
    }
}

fn push_var_like_anchor(span: Span, out: &mut Vec<EmissionAnchor>) {
    out.push(EmissionAnchor {
        start: span.offset,
        end: span_end(span),
    });
}

fn push_routine_body_anchor(span: Span, body: &FuncBody, out: &mut Vec<EmissionAnchor>) {
    out.push(EmissionAnchor {
        start: span.offset,
        end: span_end(span),
    });
    let FuncBody::Block { nested, stmts } = body;
    push_decl_anchors(nested, out);
    push_stmts(stmts, out);
}

fn push_stmt_start_offsets(stmts: &[Stmt], out: &mut Vec<usize>) {
    for stmt in stmts {
        out.push(stmt_start(stmt));
        push_nested_stmt_start_offsets(stmt, out);
    }
}

fn push_nested_stmt_start_offsets(stmt: &Stmt, out: &mut Vec<usize>) {
    match stmt {
        Stmt::Block(stmts, _) => push_stmt_start_offsets(stmts, out),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            push_nested_stmt_start_offsets(then_branch, out);
            if let Some(else_branch) = else_branch {
                push_nested_stmt_start_offsets(else_branch, out);
            }
        }
        Stmt::Case {
            arms, else_body, ..
        } => {
            for arm in arms {
                push_nested_stmt_start_offsets(&arm.body, out);
            }
            if let Some(stmts) = else_body {
                push_stmt_start_offsets(stmts, out);
            }
        }
        Stmt::For { body, .. } | Stmt::ForIn { body, .. } | Stmt::While { body, .. } => {
            push_nested_stmt_start_offsets(body, out);
        }
        Stmt::Repeat { body, .. } => push_stmt_start_offsets(body, out),
        _ => {}
    }
}

fn push_stmts(stmts: &[Stmt], out: &mut Vec<EmissionAnchor>) {
    for stmt in stmts {
        out.push(EmissionAnchor {
            start: stmt_start(stmt),
            end: stmt_end(stmt),
        });
        push_nested_emission_anchors(stmt, out);
    }
}

fn push_nested_emission_anchors(stmt: &Stmt, out: &mut Vec<EmissionAnchor>) {
    match stmt {
        Stmt::Block(stmts, _) => push_stmts(stmts, out),
        Stmt::If {
            then_branch,
            else_branch,
            ..
        } => {
            push_nested_emission_anchors(then_branch, out);
            if let Some(else_branch) = else_branch {
                push_nested_emission_anchors(else_branch, out);
            }
        }
        Stmt::Case {
            arms, else_body, ..
        } => {
            for arm in arms {
                push_case_arm_anchors(arm, out);
            }
            if let Some(stmts) = else_body {
                push_stmts(stmts, out);
            }
        }
        Stmt::For { body, .. } | Stmt::ForIn { body, .. } | Stmt::While { body, .. } => {
            push_nested_emission_anchors(body, out);
        }
        Stmt::Repeat { body, .. } => push_stmts(body, out),
        _ => {}
    }
}

fn push_case_arm_anchors(arm: &CaseArm, out: &mut Vec<EmissionAnchor>) {
    out.push(EmissionAnchor {
        start: arm.span.offset,
        end: span_end(arm.span),
    });
    push_nested_emission_anchors(&arm.body, out);
}

/// Returns `true` when `comment` may trail the construct ending at `anchor_end`.
#[must_use]
pub(crate) fn trailing_gap_allows(source: &str, anchor_end: usize, comment_start: usize) -> bool {
    if comment_start < anchor_end || !same_line(source, anchor_end, comment_start) {
        return false;
    }
    source
        .get(anchor_end..comment_start)
        .is_some_and(|gap| gap.chars().all(|c| c == ';' || c.is_whitespace()))
}

/// Returns `true` when `left` and `right` are on the same source line.
#[must_use]
pub(crate) fn same_line(source: &str, left: usize, right: usize) -> bool {
    let line_left = source[..left]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    let line_right = source[..right]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    line_left == line_right
}

#[cfg(test)]
mod tests {
    use super::{begin_keyword_offset, uses_keyword_offset};
    use fpas_parser::parse_compilation_unit;

    #[test]
    fn finds_uses_and_begin_keyword_offsets() {
        let source = "program T;\nuses Std.Console;\nbegin\nend.";
        let (_, errors) = parse_compilation_unit(source);
        assert!(errors.is_empty(), "{errors:?}");
        assert!(uses_keyword_offset(source).is_some());
        assert!(begin_keyword_offset(source).is_some());
    }
}
