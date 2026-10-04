//! Binding declarations shared by local statements and unit/program declarations.
//!
//! **Documentation:** `docs/pascal/language/basics/variables.md`.

use super::{Expr, TypeExpr, Visibility};
use fpas_lexer::Span;

/// A single initialized binding with an optional local type annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingDef {
    /// The declared binding name.
    pub name: String,
    /// Explicit type annotation; required on unit/program declarations.
    pub type_expr: Option<TypeExpr>,
    /// Initializer evaluated once when the declaration is reached.
    pub value: Expr,
    /// Visibility of a unit/program binding.
    pub visibility: Visibility,
    /// Source span covering the definition.
    pub span: Span,
}
