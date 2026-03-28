use super::{Decl, Stmt};
use fpas_lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum CompilationUnit {
    Program(Program),
    Unit(Unit),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub name: String,
    pub name_span: Span,
    pub uses: Vec<QualifiedId>,
    pub declarations: Vec<Decl>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub name: QualifiedId,
    pub uses: Vec<QualifiedId>,
    pub declarations: Vec<Decl>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedId {
    pub parts: Vec<String>,
    pub span: Span,
}
