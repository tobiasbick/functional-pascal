//! Explicit declaration bindings and optionally annotated statement-local bindings.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use super::{Expr, TypeExpr, Visibility};
use fpas_lexer::Span;

/// A parsed constant definition with an explicit type and initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    /// The declared constant name.
    pub name: String,
    /// The declared type.
    pub type_expr: TypeExpr,
    /// The initializer expression.
    pub value: Expr,
    /// The declaration visibility.
    pub visibility: Visibility,
    /// The source span covering the definition.
    pub span: Span,
}

/// A parsed variable definition with an explicit type and initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDef {
    /// The declared variable name.
    pub name: String,
    /// The declared type.
    pub type_expr: TypeExpr,
    /// The initializer expression.
    pub value: Expr,
    /// The declaration visibility.
    pub visibility: Visibility,
    /// The source span covering the definition.
    pub span: Span,
}

/// A statement-local `const` or `var` binding with an optional type annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalBinding {
    /// Release the acquired resource when this binding's scope exits.
    pub using: bool,
    /// The declared binding name.
    pub name: String,
    /// The written type annotation, when present.
    pub type_expr: Option<TypeExpr>,
    /// The required initializer expression.
    pub value: Expr,
    /// The source span covering the binding.
    pub span: Span,
}
