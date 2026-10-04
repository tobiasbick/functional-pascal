use super::QualifiedId;
use fpas_lexer::Span;

/// Parsed type expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A named type: `Point`, `integer`, `Std.Console.Color`.
    Named {
        /// The qualified type name.
        id: QualifiedId,
        /// Explicit type arguments in declaration order, when the type is generic.
        arguments: Vec<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// An `array of (T)` type, together with its source span.
    Array(Box<TypeExpr>, Span),
    /// A `channel of (T)` type, together with its source span.
    ///
    /// **Documentation:** `docs/pascal/language/types/channels.md`
    Channel(Box<TypeExpr>, Span),
    /// A `task of (T)` handle type whose `Std.Tasks.Wait` yields `T`, together with its source span.
    ///
    /// A bare `task` is parsed as the named type `task`, whose result type is inferred.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/task-handles.md`
    Task(Box<TypeExpr>, Span),
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
    /// `Result of (T, E)`
    Result {
        /// The type carried by an `Ok` value.
        ok_type: Box<TypeExpr>,
        /// The type carried by an `Error` value.
        err_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// `Option of (T)`
    Option {
        /// The type carried by a `Some` value.
        inner_type: Box<TypeExpr>,
        /// The source span covering the type expression.
        span: Span,
    },
    /// `dict of (K, V)`
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

impl TypeExpr {
    /// Return the complete source span of this type annotation.
    pub fn span(&self) -> Span {
        match self {
            Self::Named { span, .. }
            | Self::FunctionType { span, .. }
            | Self::ProcedureType { span, .. }
            | Self::Result { span, .. }
            | Self::Option { span, .. }
            | Self::Dict { span, .. }
            | Self::Array(_, span)
            | Self::Channel(_, span)
            | Self::Task(_, span) => *span,
        }
    }
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
