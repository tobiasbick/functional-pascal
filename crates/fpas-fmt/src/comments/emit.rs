//! Emits preserved leading comments before an anchor offset.

use super::CommentMap;
use crate::emit::Emitter;

/// Appends preserved comments for `anchor_offset`, then one blank line when any were written.
pub(crate) fn emit_leading_comments(
    emitter: &mut Emitter,
    comments: &CommentMap,
    anchor_offset: usize,
) {
    let attached = comments.comments_at(anchor_offset);
    if attached.is_empty() {
        return;
    }

    for text in attached {
        emit_comment_text(emitter, text);
    }
    emitter.blank_line();
}

fn emit_comment_text(emitter: &mut Emitter, text: &str) {
    let trimmed = text.trim_end();
    if trimmed.contains('\n') {
        emitter.write(trimmed);
        emitter.write("\n");
        return;
    }
    emitter.writeln(trimmed);
}
