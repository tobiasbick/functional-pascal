use super::{Decl, Stmt};
use fpas_lexer::Span;

/// Parsed top-level source file.
#[derive(Debug, Clone, PartialEq)]
pub enum CompilationUnit {
    /// Executable program compilation unit.
    Program(Program),
    /// Reusable unit compilation unit.
    Unit(Unit),
}

/// Parsed executable program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Program name declared by the header.
    pub name: String,
    /// Source span of the program name.
    pub name_span: Span,
    /// Units imported by the program's `uses` clause.
    pub uses: Vec<Import>,
    /// Top-level declarations in source order.
    pub declarations: Vec<Decl>,
    /// Statements in the program body.
    pub body: Vec<Stmt>,
    /// Source span of the complete program.
    pub span: Span,
}

/// Parsed reusable unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// Qualified unit name declared by the header.
    pub name: QualifiedId,
    /// Units imported by the unit's `uses` clause.
    pub uses: Vec<Import>,
    /// Unit-level declarations in source order.
    pub declarations: Vec<Decl>,
    /// Source span of the complete unit.
    pub span: Span,
}

/// Dot-separated identifier and its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedId {
    /// Identifier components in left-to-right source order.
    pub parts: Vec<String>,
    /// Source span of the complete qualified identifier.
    pub span: Span,
}

/// One imported unit with its sole source-level qualifier.
///
/// **Documentation:** `docs/pascal/program-structure/units.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// Canonical unit path components, independent of the local qualifier.
    pub parts: Vec<String>,
    /// Explicit local qualifier used to access the imported unit.
    pub alias: String,
    /// Exact source span of the imported unit path.
    pub unit_span: Span,
    /// Exact source span of the alias declaration.
    pub alias_span: Span,
    /// Source span of the complete `uses Unit as Alias;` declaration.
    pub span: Span,
}
