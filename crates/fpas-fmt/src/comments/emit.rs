//! Emits preserved comments at leading and trailing anchor positions.

use super::CommentMap;
use crate::emit::Emitter;

/// Appends leading comments for `anchor_offset`.
///
/// When `blank_after` is `true`, inserts one blank line after a non-empty leading group
/// (section-level comments before declarations and unit headers).
pub(crate) fn emit_leading_comments(
    emitter: &mut Emitter,
    comments: &CommentMap,
    anchor_offset: usize,
    blank_after: bool,
) {
    let attached = comments.leading_at(anchor_offset);
    if attached.is_empty() {
        return;
    }

    for text in attached {
        emit_comment_line(emitter, text);
    }
    if blank_after {
        emitter.blank_line();
    }
}

/// Appends same-line trailing comments for a construct starting at `anchor_start`.
pub(crate) fn emit_trailing_comments(
    emitter: &mut Emitter,
    comments: &CommentMap,
    anchor_start: usize,
) {
    for text in comments.trailing_at(anchor_start) {
        emitter.write(" ");
        emitter.write(text);
    }
    if !comments.trailing_at(anchor_start).is_empty() && !emitter.ends_with_newline() {
        emitter.write_line_end();
    }
}

/// Appends comments that followed the end of the compilation unit in source.
pub(crate) fn emit_trailing_end_comments(emitter: &mut Emitter, comments: &CommentMap) {
    for text in comments.trailing_end() {
        emit_comment_line(emitter, text);
    }
}

fn emit_comment_line(emitter: &mut Emitter, text: &str) {
    if text.contains('\n') {
        emitter.write(text);
        if !text.ends_with('\n') {
            emitter.write("\n");
        }
        return;
    }
    emitter.writeln(text);
}
