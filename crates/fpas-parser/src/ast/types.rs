use super::QualifiedId;
use fpas_lexer::Span;

/// Parsed type expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type: `Point`, `integer`, `Std.Console.Color`.
    Named {
        /// The qualified type name.
        id: QualifiedId,
        /// The source span covering the type expression.
        span: Span,
    },
    /// An `array of T` type, together with its source span.
    Array(Box<TypeExpr>, Span),
    /// A `channel of T` type, together with its source span.
    ///
    /// **Documentation:** `docs/pascal/language/types/channels.md`
    Channel(Box<TypeExpr>, Span),
    /// A function type with parameter and return types.
    FunctionType {
        /// The formal parameter declarations.
        params: Vec<FormalParam>,
        /// The function's return type.
        return_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// A procedure type with parameter types.
    ProcedureType {
        /// The formal parameter declarations.
        params: Vec<FormalParam>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// `Result of T, E`
    Result {
        /// The type carried by an `Ok` value.
        ok_type: Box<TypeExpr>,
        /// The type carried by an `Error` value.
        err_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// `Option of T`
    Option {
        /// The type carried by a `Some` value.
        inner_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// `dict of K to V`
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    Dict {
        /// The dictionary key type.
        key_type: Box<TypeExpr>,
        /// The dictionary value type.
        value_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
}

/// Parsed formal parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct FormalParam {
    /// Whether the parameter is declared `mutable`.
    pub mutable: bool,
    /// The parameter name.
    pub name: String,
    /// The parameter type.
    pub type_expr: TypeExpr,
    /// The source span covering the parameter declaration.
    pub span: Span,
}
