use super::{FormalParam, FuncBody, TypeExpr};
use fpas_lexer::Span;

impl Expr {
    /// Returns the source span that covers this expression.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Integer(_, span)
            | Self::Real(_, span)
            | Self::Str(_, span)
            | Self::Bool(_, span)
            | Self::Paren(_, span)
            | Self::ArrayLiteral(_, span)
            | Self::DictLiteral(_, span)
            | Self::ResultOk(_, span)
            | Self::ResultError(_, span)
            | Self::OptionSome(_, span)
            | Self::OptionNone(span)
            | Self::Nil(span)
            | Self::Try(_, span)
            | Self::Go(_, span)
            | Self::Error(span) => *span,
            Self::Designator(d) => d.span,
            Self::Call { span, .. }
            | Self::UnaryOp { span, .. }
            | Self::BinaryOp { span, .. }
            | Self::RecordLiteral { span, .. }
            | Self::RecordUpdate { span, .. }
            | Self::Postfix { span, .. } => *span,
            Self::Closure(closure) => closure.span,
        }
    }
}

/// Parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal and its source span.
    Integer(i64, Span),
    /// Real-number literal and its source span.
    Real(f64, Span),
    /// String literal contents and its source span.
    Str(String, Span),
    /// Boolean literal and its source span.
    Bool(bool, Span),
    /// Variable, field, or indexed-value designator.
    Designator(Designator),
    /// Direct call through a designator.
    Call {
        /// Designator that identifies the callable value.
        designator: Designator,
        /// Arguments in source order.
        args: Vec<Expr>,
        /// Source span of the complete call expression.
        span: Span,
    },
    /// Unary operator application.
    UnaryOp {
        /// Applied unary operator.
        op: UnaryOp,
        /// Operand of the operator.
        operand: Box<Expr>,
        /// Source span of the complete unary expression.
        span: Span,
    },
    /// Binary operator application.
    BinaryOp {
        /// Applied binary operator.
        op: BinaryOp,
        /// Left-hand operand.
        left: Box<Expr>,
        /// Right-hand operand.
        right: Box<Expr>,
        /// Source span of the complete binary expression.
        span: Span,
    },
    /// Parenthesized expression and its source span.
    Paren(Box<Expr>, Span),
    /// Array literal elements and the literal's source span.
    ArrayLiteral(Vec<Expr>, Span),
    /// Dict literal: `['key': value, ...]` or `[:]` for empty dict.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    DictLiteral(Vec<(Expr, Expr)>, Span),
    /// Record literal with explicitly initialized fields.
    RecordLiteral {
        /// Field initializers in source order.
        fields: Vec<FieldInit>,
        /// Source span of the complete record literal.
        span: Span,
    },
    /// `Ok(expr)` — wrap value in Result::Ok.
    ResultOk(Box<Expr>, Span),
    /// `Error(expr)` — wrap value in Result::Error.
    ResultError(Box<Expr>, Span),
    /// `Some(expr)` — wrap value in Option::Some.
    OptionSome(Box<Expr>, Span),
    /// `None` — Option::None literal.
    OptionNone(Span),
    /// `nil` — clears an event handler (valid only on event assignment RHS).
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    Nil(Span),
    /// `try expr` — unwrap Result/Option or propagate error.
    Try(Box<Expr>, Span),

    /// `go expr` — spawn a concurrent task.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Go(Box<Expr>, Span),
    /// `base with Field := Value; … end` — record update expression.
    ///
    /// Creates a new record by copying all fields from `base`, then overriding
    /// those listed in `fields`. The original value is unchanged.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-update.md`
    RecordUpdate {
        /// Record expression whose fields are copied.
        base: Box<Expr>,
        /// Replacement field initializers in source order.
        fields: Vec<FieldInit>,
        /// Source span of the complete update expression.
        span: Span,
    },
    /// Primary expression followed by one or more postfix suffixes.
    ///
    /// Emitted only when at least one `.Field`, `[Index]`, or `.Method(args)` follows a
    /// completed primary atom (for example a call). Ordinary designators keep
    /// [`Expr::Designator`] / [`Expr::Call`].
    ///
    /// **Documentation:** `docs/pascal/language/functions/README.md`
    Postfix {
        /// Primary expression on which the suffixes operate.
        base: Box<Expr>,
        /// Postfix operations in evaluation order.
        operations: Vec<PostfixOperation>,
        /// Source span of the complete postfix chain.
        span: Span,
    },
    /// Anonymous function or procedure expression (capturing closure).
    ///
    /// Parameter and result annotations are mandatory. The final `end` belongs to the
    /// expression; surrounding syntax supplies any separator.
    ///
    /// **Documentation:** `docs/pascal/language/functions/closures.md`
    Closure(Box<ClosureExpr>),
    /// Placeholder emitted when the parser fails to parse an expression.
    /// Downstream passes should propagate this as an error rather than
    /// checking or compiling it.
    Error(Span),
}

/// Payload for [`Expr::Closure`].
///
/// Kept behind a box so [`Expr`] does not grow large enough to trip
/// `clippy::large_enum_variant` on dependent enums such as [`super::Stmt`].
///
/// **Documentation:** `docs/pascal/language/functions/closures.md`
#[derive(Debug, Clone, PartialEq)]
pub struct ClosureExpr {
    /// `true` for `function(...) : T … end`, `false` for `procedure(...) … end`.
    pub is_function: bool,
    /// Formal parameters in declaration order.
    pub params: Vec<FormalParam>,
    /// Declared result type for a function closure, or `None` for a procedure closure.
    pub return_type: Option<TypeExpr>,
    /// Parsed body of the closure.
    pub body: FuncBody,
    /// Source span of the complete closure expression.
    pub span: Span,
}

/// One suffix in an [`Expr::Postfix`] chain.
///
/// **Documentation:** `docs/pascal/language/functions/README.md`
#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOperation {
    /// `.Field` access on the preceding value.
    Field {
        /// Accessed field name.
        name: String,
        /// Source span of this suffix.
        span: Span,
    },
    /// `[Index]` access on the preceding value.
    Index {
        /// Index expression enclosed by the brackets.
        index: Box<Expr>,
        /// Source span of this suffix.
        span: Span,
    },
    /// `.Method(args)` instance call on the preceding value.
    MethodCall {
        /// Called method name.
        name: String,
        /// Arguments in source order.
        args: Vec<Expr>,
        /// Source span of this suffix.
        span: Span,
    },
}

/// Record or `new` field initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    /// Initialized field name.
    pub name: String,
    /// Expression that supplies the field value.
    pub value: Expr,
    /// Source span of the complete initializer.
    pub span: Span,
}

/// Parsed variable/field/index access path.
#[derive(Debug, Clone, PartialEq)]
pub struct Designator {
    /// Path segments in left-to-right source order.
    pub parts: Vec<DesignatorPart>,
    /// Source span of the complete designator.
    pub span: Span,
}

/// One segment in a parsed designator path.
#[derive(Debug, Clone, PartialEq)]
pub enum DesignatorPart {
    /// Identifier segment and its source span.
    Ident(String, Span),
    /// Index expression and the bracketed segment's source span.
    Index(Expr, Span),
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Logical negation with `not`.
    Not,
    /// Arithmetic negation with unary `-`.
    Negate,
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    /// Multiplication with `*`.
    Mul,
    /// Real division with `/`.
    RealDiv,
    /// Integer division with `div`.
    IntDiv,
    /// Integer remainder with `mod`.
    Mod,
    /// Logical or bitwise conjunction with `and`.
    And,
    /// Left shift with `shl`.
    Shl,
    /// Right shift with `shr`.
    Shr,
    /// Addition with `+`.
    Add,
    /// Subtraction with `-`.
    Sub,
    /// Logical or bitwise disjunction with `or`.
    Or,
    /// Logical or bitwise exclusive disjunction with `xor`.
    Xor,
    /// Equality comparison with `=`.
    Eq,
    /// Inequality comparison with `<>`.
    NotEq,
    /// Less-than comparison with `<`.
    Lt,
    /// Greater-than comparison with `>`.
    Gt,
    /// Less-than-or-equal comparison with `<=`.
    LtEq,
    /// Greater-than-or-equal comparison with `>=`.
    GtEq,
    /// Membership test with `in`.
    In,
}
