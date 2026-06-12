//! Shared source-span helpers for comment anchoring and emission.

use fpas_parser::{Decl, FunctionDecl, ProcedureDecl};

/// Returns the lexer anchor offset for a top-level or nested declaration.
#[must_use]
pub(crate) fn decl_span(decl: &Decl) -> usize {
    match decl {
        Decl::Const(def) => def.span.offset,
        Decl::Var(def) | Decl::MutableVar(def) => def.span.offset,
        Decl::TypeDef(def) => def.span.offset,
        Decl::Function(FunctionDecl { span, .. }) | Decl::Procedure(ProcedureDecl { span, .. }) => {
            span.offset
        }
    }
}
