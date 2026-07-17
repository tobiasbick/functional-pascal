use super::{FormalParam, FuncBody, TypeExpr};
use fpas_lexer::Span;

impl Expr {
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
    Integer(i64, Span),
    Real(f64, Span),
    Str(String, Span),
    Bool(bool, Span),
    Designator(Designator),
    Call {
        designator: Designator,
        args: Vec<Expr>,
        span: Span,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Paren(Box<Expr>, Span),
    ArrayLiteral(Vec<Expr>, Span),
    /// Dict literal: `['key': value, ...]` or `[:]` for empty dict.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    DictLiteral(Vec<(Expr, Expr)>, Span),
    RecordLiteral {
        fields: Vec<FieldInit>,
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
        base: Box<Expr>,
        fields: Vec<FieldInit>,
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
        base: Box<Expr>,
        operations: Vec<PostfixOperation>,
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
    pub params: Vec<FormalParam>,
    pub return_type: Option<TypeExpr>,
    pub body: FuncBody,
    pub span: Span,
}

/// One suffix in an [`Expr::Postfix`] chain.
///
/// **Documentation:** `docs/pascal/language/functions/README.md`
#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOperation {
    /// `.Field` access on the preceding value.
    Field { name: String, span: Span },
    /// `[Index]` access on the preceding value.
    Index { index: Box<Expr>, span: Span },
    /// `.Method(args)` instance call on the preceding value.
    MethodCall {
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
}

/// Record or `new` field initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

/// Parsed variable/field/index access path.
#[derive(Debug, Clone, PartialEq)]
pub struct Designator {
    pub parts: Vec<DesignatorPart>,
    pub span: Span,
}

/// One segment in a parsed designator path.
#[derive(Debug, Clone, PartialEq)]
pub enum DesignatorPart {
    Ident(String, Span),
    Index(Expr, Span),
}

/// Unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
}

/// Binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    RealDiv,
    IntDiv,
    Mod,
    And,
    Shl,
    Shr,
    Add,
    Sub,
    Or,
    Xor,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    In,
}
