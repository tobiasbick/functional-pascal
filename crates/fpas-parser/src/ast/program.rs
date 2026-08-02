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
    pub uses: Vec<QualifiedId>,
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
    pub uses: Vec<QualifiedId>,
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
