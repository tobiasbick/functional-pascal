//! Structural layout for calls, parentheses, and collection literals.

use super::super::width;
use super::{Emitter, emit_expr};
use crate::comments::{CommentMap, emit_leading_comments, emit_trailing_comments};
use crate::style::INDENT_WIDTH;
use fpas_lexer::Span;
use fpas_parser::{DesignatorPart, Expr, FuncBody, PostfixOperation};

/// Emits a complete argument list, including its parentheses.
pub(crate) fn emit_arguments(
    emitter: &mut Emitter,
    args: &[Expr],
    span: Span,
    comments: &CommentMap,
) {
    let group = emitter.layout.group(span);
    // A sole collection argument shares the call's brackets: Builder([ ... ]).
    let compact_collection = matches!(args, [Expr::ArrayLiteral(..) | Expr::DictLiteral(..)])
        && group.is_some_and(|g| {
            !emitter.layout.break_before(args[0].span().offset)
                && !emitter.layout.break_before(g.close)
        });
    emit_list(
        emitter,
        args,
        ListStyle {
            span,
            open: "(",
            close: ")",
            compact_collection,
        },
        Expr::span,
        args.iter().any(|arg| expression_multiline(arg, comments)),
        comments,
        |inner, arg| emit_expr(inner, arg, 0, comments),
    );
}

/// Emits a bracketed index expression with the same source layout as other lists.
pub(super) fn emit_index(emitter: &mut Emitter, index: &Expr, span: Span, comments: &CommentMap) {
    emit_list(
        emitter,
        std::slice::from_ref(index),
        ListStyle {
            span,
            open: "[",
            close: "]",
            compact_collection: false,
        },
        Expr::span,
        expression_multiline(index, comments),
        comments,
        |inner, expr| emit_expr(inner, expr, 0, comments),
    );
}

/// Delimiters and the optional sole-collection argument exception.
pub(super) struct ListStyle<'a> {
    pub(super) span: Span,
    pub(super) open: &'a str,
    pub(super) close: &'a str,
    pub(super) compact_collection: bool,
}

/// Emits list items while retaining the author's comma-separated line groups.
pub(super) fn emit_list<T>(
    emitter: &mut Emitter,
    items: &[T],
    style: ListStyle<'_>,
    span_of: impl Fn(&T) -> Span,
    force_multiline: bool,
    comments: &CommentMap,
    emit_item: impl Fn(&mut Emitter, &T),
) {
    let group = emitter.layout.group(style.span);
    let has_comments = list_comments(
        items.iter().map(|item| span_of(item).offset),
        group,
        comments,
    );
    let source_multiline = has_comments
        || (!style.compact_collection
            && (force_multiline
                || emitter
                    .layout
                    .list_breaks(group, items.iter().map(|item| span_of(item).offset))));
    let widths: Vec<_> = if emitter.measuring {
        Vec::new()
    } else {
        items
            .iter()
            .map(|item| width::measure(|flat| emit_item(flat, item)))
            .collect()
    };
    let inline_width = widths
        .iter()
        .try_fold(0, |total, item| item.map(|width| total + width))
        .map(|width| width + items.len().saturating_sub(1) * 2);
    let auto_multiline = !items.is_empty()
        && !style.compact_collection
        && emitter.would_exceed_width(
            inline_width,
            width::text_width(style.open) + width::text_width(style.close) + 1,
        );
    let multiline = source_multiline || auto_multiline;
    let one_per_line = auto_multiline && !source_multiline;
    let base = emitter.line_indent();
    let item_indent = base + INDENT_WIDTH;
    emitter.write(style.open);
    if !multiline {
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                emitter.write(", ");
            }
            emit_item(emitter, item);
        }
        emitter.write(style.close);
        return;
    }
    emitter.write_line_end();
    emitter.with_suffix_width(0, |inner| {
        inner.at_indent(item_indent, |inner| {
            for (index, item) in items.iter().enumerate() {
                let offset = span_of(item).offset;
                let row_width = if inner.measuring {
                    None
                } else {
                    Some(width::first_line(comments, |flat| emit_item(flat, item)))
                };
                let new_line = index == 0
                    || one_per_line
                    || inner.layout.break_before(offset)
                    || !comments.leading_at(offset).is_empty()
                    || inner.ends_with_newline()
                    || inner.would_exceed_width(row_width, 2);
                if new_line {
                    if !inner.ends_with_newline() {
                        inner.write_line_end();
                    }
                    emit_leading_comments(inner, comments, offset, false);
                    inner.write_current_indent();
                } else {
                    inner.write(" ");
                }
                emit_item(inner, item);
                if index + 1 < items.len() {
                    inner.write(",");
                }
                emit_trailing_comments(inner, comments, offset);
            }
            if let Some(group) = group {
                if !inner.ends_with_newline() {
                    inner.write_line_end();
                }
                emit_leading_comments(inner, comments, group.close, false);
            }
        })
    });
    if !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
    emitter.at_indent(base, |inner| {
        inner.write_current_indent();
        inner.write(style.close);
    });
}

fn list_comments(
    offsets: impl Iterator<Item = usize>,
    group: Option<crate::layout::Group>,
    comments: &CommentMap,
) -> bool {
    offsets.into_iter().any(|offset| {
        !comments.leading_at(offset).is_empty() || !comments.trailing_at(offset).is_empty()
    }) || group.is_some_and(|g| !comments.leading_at(g.close).is_empty())
}

fn expression_list_multiline(items: &[Expr], span: Span, comments: &CommentMap) -> bool {
    let group = comments.layout.group(span);
    comments
        .layout
        .list_breaks(group, items.iter().map(|item| item.span().offset))
        || list_comments(items.iter().map(|item| item.span().offset), group, comments)
        || items
            .iter()
            .any(|item| expression_multiline(item, comments))
}

/// Predicts emitted line breaks, excluding discarded whitespace before postfix suffixes.
pub(super) fn expression_multiline(expr: &Expr, comments: &CommentMap) -> bool {
    match expr {
        Expr::Case { .. } => true,
        Expr::RecordUpdate { base, fields, span } => {
            fields.iter().any(|field| {
                !comments.leading_at(field.span.offset).is_empty()
                    || !comments.trailing_at(field.span.offset).is_empty()
            }) || comments.closer_anchor(span.offset).is_some_and(|offset| {
                !comments.leading_at(offset).is_empty() || !comments.trailing_at(offset).is_empty()
            }) || expression_multiline(base, comments)
                || fields
                    .iter()
                    .any(|field| expression_multiline(&field.value, comments))
        }
        Expr::Closure(closure) => {
            let FuncBody::Block { nested, stmts } = &closure.body;
            !nested.is_empty()
                || !stmts.is_empty()
                || [
                    comments.body_anchor(closure.span.offset),
                    comments.closer_anchor(closure.span.offset),
                ]
                .into_iter()
                .flatten()
                .any(|offset| {
                    !comments.leading_at(offset).is_empty()
                        || !comments.trailing_at(offset).is_empty()
                })
                || closure
                    .params
                    .first()
                    .zip(closure.params.last())
                    .and_then(|(first, last)| comments.layout.enclosing(first.span, last.span))
                    .is_some_and(|group| comments.layout.breaks_between(group.open, group.close))
        }
        Expr::Call { args, span, .. } | Expr::ArrayLiteral(args, span) => {
            expression_list_multiline(args, *span, comments)
        }
        Expr::DictLiteral(pairs, span) => {
            let group = comments.layout.group(*span);
            comments
                .layout
                .list_breaks(group, pairs.iter().map(|(key, _)| key.span().offset))
                || list_comments(
                    pairs.iter().map(|(key, _)| key.span().offset),
                    group,
                    comments,
                )
                || pairs.iter().any(|(key, value)| {
                    expression_multiline(key, comments) || expression_multiline(value, comments)
                })
        }
        Expr::Designator(designator) | Expr::VarArgument { designator, .. } => {
            designator.parts.iter().any(|part| match part {
                DesignatorPart::Index(index, span) => {
                    expression_list_multiline(std::slice::from_ref(index), *span, comments)
                }
                DesignatorPart::Ident(..) => false,
            })
        }
        Expr::Paren(operand, span)
        | Expr::ResultOk(operand, span)
        | Expr::ResultError(operand, span)
        | Expr::OptionSome(operand, span) => {
            expression_list_multiline(std::slice::from_ref(operand.as_ref()), *span, comments)
        }
        Expr::UnaryOp { operand, .. }
        | Expr::Try(operand, _)
        | Expr::Go(operand, _)
        | Expr::NamedArgument { value: operand, .. } => expression_multiline(operand, comments),
        Expr::BinaryOp { left, right, .. } => {
            comments.layout.breaks_between(
                left.span()
                    .offset
                    .saturating_add(left.span().length)
                    .saturating_sub(1),
                right.span().offset,
            ) || expression_multiline(left, comments)
                || expression_multiline(right, comments)
        }
        Expr::Postfix {
            base, operations, ..
        } => {
            expression_multiline(base, comments)
                || operations.iter().any(|op| match op {
                    PostfixOperation::Index { index, span } => expression_list_multiline(
                        std::slice::from_ref(index.as_ref()),
                        *span,
                        comments,
                    ),
                    PostfixOperation::MethodCall { args, span, .. } => {
                        expression_list_multiline(args, *span, comments)
                    }
                    PostfixOperation::Field { .. } => false,
                })
        }
        Expr::If {
            branches,
            else_value,
            else_span,
            span,
        } => super::conditional::needs_multiline(branches, else_value, *else_span, *span, comments),
        Expr::Is { value, .. } => expression_multiline(value, comments),
        _ => false,
    }
}
