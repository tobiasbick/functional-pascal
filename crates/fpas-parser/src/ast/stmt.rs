use super::{Designator, Expr, TypeExpr, VarDef};
use fpas_lexer::Span;

/// Parsed statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Compound `begin ... end` statement and its source span.
    Block(Vec<Stmt>, Span),
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
    /// Panic statement, its payload expression, and its source span.
    Panic(Expr, Span),
    /// Conditional `if` statement.
    If {
        /// Boolean condition that selects a branch.
        condition: Expr,
        /// Statement executed when the condition is true.
        then_branch: Box<Stmt>,
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

/// One arm of a [`Stmt::Case`] statement.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    /// Labels that select this arm.
    pub labels: Vec<CaseLabel>,
    /// Optional condition evaluated after a label matches.
    pub guard: Option<Expr>,
    /// Statement executed when a label and the optional guard match.
    pub body: Stmt,
    /// Source span of the complete case arm.
    pub span: Span,
}

/// Label that selects a [`CaseArm`].
#[derive(Debug, Clone, PartialEq)]
pub enum CaseLabel {
    /// Classic value label: single value or range (`1`, `1..10`).
    Value {
        /// Single label expression or inclusive range start.
        start: Expr,
        /// Inclusive range end, or `None` for a single value.
        end: Option<Expr>,
        /// Source span of the complete label.
        span: Span,
    },
    /// Destructure pattern for Result/Option: `Ok(Binding)`, `Error(Binding)`, `Some(Binding)`, `None`.
    Destructure {
        /// Result or option variant matched by the pattern.
        variant: DestructureVariant,
        /// Name bound to the wrapped value, when the variant carries one.
        binding: Option<String>,
        /// Source span of the complete pattern.
        span: Span,
    },
}

/// Result or option variant used by a destructuring case label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructureVariant {
    /// Successful result containing a value.
    Ok,
    /// Failed result containing an error value.
    Error,
    /// Present option containing a value.
    Some,
    /// Empty option without a value.
    None,
}
