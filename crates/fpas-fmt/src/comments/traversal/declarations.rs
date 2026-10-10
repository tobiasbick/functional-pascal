//! Declaration, member, routine, and named-closer comment anchors.

use super::super::anchors::span_end;
use super::expressions::collect_expr;
use super::{CollectedAnchors, collect_body, collect_header, collect_stmts, push_span};
use fpas_lexer::Span;
use fpas_parser::{Decl, FuncBody, RecordMethod, TypeBody};

/// Collects declaration and member anchors, including named endings.
pub(super) fn collect_decls(decls: &[Decl], begins: &[usize], out: &mut CollectedAnchors) {
    for decl in decls {
        let start = crate::span::decl_span(decl);
        out.leading.push(start);
        out.declarations.insert(start);
        match decl {
            Decl::Const(def) => {
                push_span(def.span, out);
                collect_expr(&def.value, begins, out);
            }
            Decl::Var(def) => {
                push_span(def.span, out);
                collect_expr(&def.value, begins, out);
            }
            Decl::TypeDef(def) => {
                push_span(def.span, out);
                match &def.body {
                    TypeBody::Record(record) => {
                        collect_closer(record.span, out);
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
                    }
                    TypeBody::Enum(enum_type) => {
                        collect_closer(enum_type.span, out);
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
    collect_closer(span, out);
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

/// Associates the last named ending in a block span with its owner.
pub(super) fn collect_closer(span: Span, out: &mut CollectedAnchors) {
    let upper = out
        .named_ends
        .partition_point(|anchor| anchor.end <= span_end(span));
    if let Some(anchor) = upper
        .checked_sub(1)
        .and_then(|index| out.named_ends.get(index))
        .copied()
        .filter(|anchor| anchor.start >= span.offset)
    {
        out.closers.insert(span.offset, anchor.start);
        out.leading.push(anchor.start);
    }
}
