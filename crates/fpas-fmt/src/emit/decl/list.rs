//! Ordered declarations with complete keywords and explicit visibility.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md`

use fpas_parser::Decl;

use crate::comments::CommentMap;

use super::super::Emitter;
use super::item::emit_decl;

/// Emit individual declarations, keeping related simple bindings adjacent.
pub(crate) fn emit_decls(emitter: &mut Emitter, declarations: &[Decl], comments: &CommentMap) {
    for (index, declaration) in declarations.iter().enumerate() {
        if index > 0 && needs_blank_line(&declarations[index - 1], declaration) {
            emitter.blank_line();
        }
        emit_decl(emitter, declaration, comments);
    }
}

fn needs_blank_line(previous: &Decl, next: &Decl) -> bool {
    !matches!(
        (previous, next),
        (Decl::Const(_), Decl::Const(_))
            | (Decl::Var(_), Decl::Var(_))
            | (Decl::MutableVar(_), Decl::MutableVar(_))
    ) || previous.visibility() != next.visibility()
}
