use super::{Designator, Expr, TypeExpr, VarDef};
use fpas_lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>, Span),
    Var(VarDef),
    MutableVar(VarDef),
    Assign {
        target: Designator,
        value: Expr,
        span: Span,
    },
    Return(Option<Expr>, Span),
    Panic(Expr, Span),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
        span: Span,
    },
    Case {
        expr: Expr,
        arms: Vec<CaseArm>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    For {
        var_name: String,
        var_type: TypeExpr,
        start: Expr,
        direction: ForDirection,
        end: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    ForIn {
        var_name: String,
        var_type: TypeExpr,
        iterable: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
        span: Span,
    },
    Repeat {
        body: Vec<Stmt>,
        condition: Expr,
        span: Span,
    },
    Break(Span),
    Continue(Span),
    Call {
        designator: Designator,
        args: Vec<Expr>,
        span: Span,
    },
    /// `go` statement: spawn a concurrent task.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Go {
        expr: Expr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForDirection {
    To,
    Downto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub labels: Vec<CaseLabel>,
    pub guard: Option<Expr>,
    pub body: Stmt,
    pub span: Span,
}

#[expect(
    clippy::large_enum_variant,
    reason = "case labels keep expression payloads inline because the parser AST is shared across parser, sema, linker, and compiler passes"
)]
#[derive(Debug, Clone, PartialEq)]
pub enum CaseLabel {
    /// Classic value label: single value or range (`1`, `1..10`).
    Value {
        start: Expr,
        end: Option<Expr>,
        span: Span,
    },
    /// Destructure pattern for Result/Option: `Ok(Binding)`, `Error(Binding)`, `Some(Binding)`, `None`.
    Destructure {
        variant: DestructureVariant,
        binding: Option<String>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructureVariant {
    Ok,
    Error,
    Some,
    None,
}

