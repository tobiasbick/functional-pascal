use super::QualifiedId;
use fpas_lexer::Span;

/// Parsed type expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type, optionally with type arguments: `Point` or `Stack of integer`.
    Named {
        id: QualifiedId,
        /// Generic type arguments supplied via `of`: `Stack of integer`.
        type_args: Vec<TypeExpr>,
        span: Span,
    },
    Array(Box<TypeExpr>, Span),
    FunctionType {
        params: Vec<FormalParam>,
        return_type: Box<TypeExpr>,
        span: Span,
    },
    ProcedureType {
        params: Vec<FormalParam>,
        span: Span,
    },
    /// `Result of T, E`
    Result {
        ok_type: Box<TypeExpr>,
        err_type: Box<TypeExpr>,
        span: Span,
    },
    /// `Option of T`
    Option {
        inner_type: Box<TypeExpr>,
        span: Span,
    },
    /// `channel of T`
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Channel {
        inner_type: Box<TypeExpr>,
        span: Span,
    },
    /// `dict of K to V`
    ///
    /// **Documentation:** `docs/future/advanced-types.md`
    Dict {
        key_type: Box<TypeExpr>,
        value_type: Box<TypeExpr>,
        span: Span,
    },
}

/// Parsed formal parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct FormalParam {
    pub mutable: bool,
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}
