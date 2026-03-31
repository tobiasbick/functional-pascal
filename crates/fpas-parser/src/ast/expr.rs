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
            | Self::New { span, .. }
            | Self::Function { span, .. }
            | Self::RecordUpdate { span, .. } => *span,
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
    /// **Documentation:** `docs/future/advanced-types.md`
    DictLiteral(Vec<(Expr, Expr)>, Span),
    RecordLiteral {
        fields: Vec<FieldInit>,
        span: Span,
    },
    /// `new T with Field := Value; ... end`
    ///
    /// **Documentation:** `docs/pascal/05-types.md`
    New {
        type_expr: TypeExpr,
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
    /// Anonymous function expression (lambda / closure).
    /// `function(Params): ReturnType begin Stmts end`
    ///
    /// **Documentation:** `docs/future/closures.md`
    Function {
        params: Vec<FormalParam>,
        return_type: TypeExpr,
        body: FuncBody,
        span: Span,
    },
    /// `go expr` — spawn a concurrent task.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Go(Box<Expr>, Span),
    /// `base with Field := Value; … end` — record update expression.
    ///
    /// Creates a new record by copying all fields from `base`, then overriding
    /// those listed in `fields`. The original value is unchanged.
    ///
    /// **Documentation:** `docs/pascal/05-types.md` (Record Update Expression)
    RecordUpdate {
        base: Box<Expr>,
        fields: Vec<FieldInit>,
        span: Span,
    },
    /// Placeholder emitted when the parser fails to parse an expression.
    /// Downstream passes should propagate this as an error rather than
    /// checking or compiling it.
    Error(Span),
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
}
