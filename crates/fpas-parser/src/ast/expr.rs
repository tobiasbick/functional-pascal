use super::{FormalParam, FuncBody, Pattern, TypeExpr};
use fpas_lexer::Span;

impl Expr {
    /// Appends the conditions of a top-level `and` chain in evaluation order.
    ///
    /// Parentheses and other operators remain single conditions. Semantic checking
    /// and lowering use this same boundary for pattern-test bindings.
    /// **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`
    pub fn collect_conjuncts<'a>(&'a self, out: &mut Vec<&'a Self>) {
        match self {
            Self::BinaryOp {
                op: BinaryOp::And,
                left,
                right,
                ..
            } => {
                left.collect_conjuncts(out);
                right.collect_conjuncts(out);
            }
            other => out.push(other),
        }
    }

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
            | Self::Try(_, span)
            | Self::Go(_, span)
            | Self::Error(span) => *span,
            Self::Designator(d) => d.span,
            Self::Call { span, .. }
            | Self::UnaryOp { span, .. }
            | Self::BinaryOp { span, .. }
            | Self::RecordUpdate { span, .. }
            | Self::Postfix { span, .. }
            | Self::NamedArgument { span, .. }
            | Self::VarArgument { span, .. }
            | Self::Is { span, .. }
            | Self::If { span, .. }
            | Self::Case { span, .. } => *span,
            Self::Closure(closure) => closure.span,
        }
    }

    /// Returns the value of a call argument, looking through a named argument.
    ///
    /// **Documentation:** `docs/pascal/language/functions/parameters.md`
    #[must_use]
    pub fn argument_value(&self) -> &Expr {
        match self {
            Self::NamedArgument { value, .. } => value,
            other => other,
        }
    }

    /// Returns the written parameter name of a named call argument.
    #[must_use]
    pub fn argument_name(&self) -> Option<&str> {
        match self {
            Self::NamedArgument { name, .. } => Some(name),
            _ => None,
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
    /// `Name := Value` or `Name := var Designator` argument of a fully named call.
    ///
    /// Appears only directly inside a call argument list; semantic analysis maps
    /// the name to a declared parameter.
    ///
    /// **Documentation:** `docs/pascal/language/functions/parameters.md`
    NamedArgument {
        /// Parameter name as written.
        name: String,
        /// Source span of the parameter name.
        name_span: Span,
        /// Argument value, including a `VarArgument` for a reference parameter.
        value: Box<Expr>,
        /// Source span of the complete named argument.
        span: Span,
    },
    /// `var Designator` argument for a `var` parameter.
    ///
    /// Appears in a call argument list, directly or as a named argument's value.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    VarArgument {
        /// Caller variable, field, or element passed by reference.
        designator: Designator,
        /// Source span of the complete argument including `var`.
        span: Span,
    },
    /// `Value is Pattern`: tests one pattern and binds its names.
    ///
    /// Valid only as an `if`, `elsif`, or `while` condition, optionally joined
    /// with later conditions by `and`.
    ///
    /// **Documentation:** `docs/pascal/language/pattern-matching/is-test.md`
    Is {
        /// Tested value.
        value: Box<Expr>,
        /// Pattern the value must match.
        pattern: Box<Pattern>,
        /// Source span of the complete test.
        span: Span,
    },
    /// `if C then A elsif D then B else E end if` producing one of its branch values.
    ///
    /// **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`
    If {
        /// The `if` branch followed by each `elsif` branch, in source order.
        branches: Vec<IfExprBranch>,
        /// Value of the required `else` branch.
        else_value: Box<Expr>,
        /// Source span from `else` through its value.
        else_span: Span,
        /// Source span of the complete expression including `end if`.
        span: Span,
    },
    /// `case Value of when Labels [if Guard]: Expression; ... [else Expression;] end case`
    /// producing the value of the first matching arm.
    ///
    /// **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`
    Case {
        /// Value matched by the arms.
        selector: Box<Expr>,
        /// Arms in source order.
        arms: Vec<CaseExprArm>,
        /// Optional `else` arm for selectors whose values are not all listed.
        else_arm: Option<Box<CaseExprElse>>,
        /// Source span of the complete expression including `end case`.
        span: Span,
    },
    /// Placeholder emitted when the parser fails to parse an expression.
    /// Downstream passes should propagate this as an error rather than
    /// checking or compiling it.
    Error(Span),
}

/// One condition and its value in an [`Expr::If`].
///
/// **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`
#[derive(Debug, Clone, PartialEq)]
pub struct IfExprBranch {
    /// Boolean condition, which may contain `is` pattern tests.
    pub condition: Expr,
    /// Value produced when the condition holds.
    pub value: Expr,
    /// Source span from `if` or `elsif` through the value.
    pub span: Span,
}

/// One `when` arm of an [`Expr::Case`].
///
/// **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`
#[derive(Debug, Clone, PartialEq)]
pub struct CaseExprArm {
    /// Labels that select this arm.
    pub labels: Vec<super::CaseLabel>,
    /// Optional condition evaluated after a label matches.
    pub guard: Option<Expr>,
    /// Value produced when a label and the optional guard match.
    pub value: Expr,
    /// Source span from `when` through the value.
    pub span: Span,
}

/// The `else` arm of an [`Expr::Case`].
#[derive(Debug, Clone, PartialEq)]
pub struct CaseExprElse {
    /// Value produced when no arm matches.
    pub value: Expr,
    /// Source span from `else` through the value.
    pub span: Span,
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

/// Field initializer of a record update.
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
    /// Boolean conjunction with `and`.
    And,
    /// Addition with `+`.
    Add,
    /// Subtraction with `-`.
    Sub,
    /// Boolean disjunction with `or`.
    Or,
    /// Boolean exclusive disjunction with `xor`.
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
