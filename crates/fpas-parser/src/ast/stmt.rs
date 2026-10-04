use super::{Designator, Expr, TypeExpr, VarDef};
use fpas_lexer::Span;

impl Stmt {
    /// Returns the source span that covers this statement.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Block(_, span)
            | Self::StatementList(_, span)
            | Self::Null(span)
            | Self::Return(_, span)
            | Self::Panic(_, span)
            | Self::Discard(_, span)
            | Self::Break(span)
            | Self::Continue(span) => *span,
            Self::Var(value) | Self::MutableVar(value) => value.span,
            Self::Assign { span, .. }
            | Self::If { span, .. }
            | Self::Case { span, .. }
            | Self::For { span, .. }
            | Self::ForIn { span, .. }
            | Self::While { span, .. }
            | Self::Repeat { span, .. }
            | Self::Call { span, .. }
            | Self::Expression { span, .. }
            | Self::Go { span, .. } => *span,
        }
    }
}

/// Parsed statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Compound `begin ... end` statement and its source span.
    Block(Vec<Stmt>, Span),
    /// A named control-flow body's statement list, without an explicit lexical block.
    StatementList(Vec<Stmt>, Span),
    /// Explicit no-action statement `null;`.
    Null(Span),
    /// Immutable local variable declaration.
    Var(VarDef),
    /// Mutable local variable declaration.
    MutableVar(VarDef),
    /// Assignment to a variable, field, or indexed element.
    Assign {
        /// Designator that receives the assigned value.
        target: Designator,
        /// Expression whose value is assigned.
        value: Expr,
        /// Source span of the complete assignment.
        span: Span,
    },
    /// Return statement, its optional result expression, and its source span.
    Return(Option<Expr>, Span),
    /// Evaluate a value exactly once and explicitly discard its result.
    Discard(Expr, Span),
    /// Panic statement, its payload expression, and its source span.
    Panic(Expr, Span),
    /// Conditional `if` statement.
    If {
        /// Boolean condition that selects a branch.
        condition: Expr,
        /// Statement executed when the condition is true.
        then_branch: Box<Stmt>,
        /// Additional condition/body pairs evaluated in written `elsif` order.
        elsif_branches: Vec<(Expr, Box<Stmt>)>,
        /// Statement executed when the condition is false, when present.
        else_branch: Option<Box<Stmt>>,
        /// Source span of the complete conditional.
        span: Span,
    },
    /// Pattern-matching `case` statement.
    Case {
        /// Scrutinee expression matched by the arms.
        expr: Expr,
        /// Case arms in source order.
        arms: Vec<CaseArm>,
        /// Statements in the optional `else` branch.
        else_body: Option<Vec<Stmt>>,
        /// Source span of the complete case statement.
        span: Span,
    },
    /// Counting `for` loop.
    For {
        /// Loop variable name.
        var_name: String,
        /// Declared type of the loop variable.
        var_type: TypeExpr,
        /// Initial value of the loop variable.
        start: Expr,
        /// Direction in which the loop variable advances.
        direction: ForDirection,
        /// Inclusive final value of the loop variable.
        end: Expr,
        /// Statement executed for each value.
        body: Box<Stmt>,
        /// Source span of the complete loop.
        span: Span,
    },
    /// Collection-iteration `for ... in` loop.
    ForIn {
        /// Loop variable name.
        var_name: String,
        /// Declared type of the loop variable.
        var_type: TypeExpr,
        /// Expression that supplies the iterated values.
        iterable: Expr,
        /// Statement executed for each value.
        body: Box<Stmt>,
        /// Source span of the complete loop.
        span: Span,
    },
    /// Precondition `while` loop.
    While {
        /// Condition evaluated before each iteration.
        condition: Expr,
        /// Statement executed while the condition is true.
        body: Box<Stmt>,
        /// Source span of the complete loop.
        span: Span,
    },
    /// Postcondition `repeat ... until` loop.
    Repeat {
        /// Statements executed before each condition check.
        body: Vec<Stmt>,
        /// Condition that terminates the loop when true.
        condition: Expr,
        /// Source span of the complete loop.
        span: Span,
    },
    /// `break` statement and its source span.
    Break(Span),
    /// `continue` statement and its source span.
    Continue(Span),
    /// Procedure call used as a statement.
    Call {
        /// Designator that identifies the called procedure.
        designator: Designator,
        /// Arguments in source order.
        args: Vec<Expr>,
        /// Source span of the complete call statement.
        span: Span,
    },
    /// A postfix chain used as a statement, ending in an instance method call.
    ///
    /// **Documentation:** `docs/pascal/language/functions/postfix-chaining.md`
    Expression {
        /// Postfix expression evaluated for its side effect.
        expr: Expr,
        /// Source span of the complete statement.
        span: Span,
    },
    /// `go` statement: spawn a concurrent task.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Go {
        /// Call expression executed as a concurrent task.
        expr: Expr,
        /// Source span of the complete `go` statement.
        span: Span,
    },
}

/// Counting direction of a [`Stmt::For`] loop.
#[derive(Debug, Clone, PartialEq)]
pub enum ForDirection {
    /// Increase the loop variable toward the inclusive bound.
    To,
    /// Decrease the loop variable toward the inclusive bound.
    Downto,
}

/// One pattern-selected statement or value-expression arm.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm<Body = Stmt> {
    /// Labels that select this arm.
    pub labels: Vec<CaseLabel>,
    /// Optional condition evaluated after a label matches.
    pub guard: Option<Expr>,
    /// Statement or expression selected after a label and optional guard match.
    pub body: Body,
    /// Source span of the complete case arm.
    pub span: Span,
}

/// Pattern that selects a [`CaseArm`], shared with value-producing decisions.
///
/// **Documentation:** `docs/pascal/language/pattern-matching/README.md`.
pub type CaseLabel = super::Pattern;
