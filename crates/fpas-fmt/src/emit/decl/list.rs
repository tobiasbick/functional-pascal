//! Individual declarations in their original source order.

use super::super::Emitter;
use super::item::emit_decl;
use crate::comments::CommentMap;
use fpas_parser::Decl;

/// Emits each declaration with its own keyword and visibility modifier.
pub(crate) fn emit_decls(emitter: &mut Emitter, declarations: &[Decl], comments: &CommentMap) {
    for (index, declaration) in declarations.iter().enumerate() {
        if index > 0
            && (matches!(
                declaration,
                Decl::Function(_) | Decl::Procedure(_) | Decl::TypeDef(_)
            ) || matches!(
                declarations[index - 1],
                Decl::Function(_) | Decl::Procedure(_) | Decl::TypeDef(_)
            ))
        {
            emitter.blank_line();
        }
        emit_decl(emitter, declaration, comments);
    }
}
