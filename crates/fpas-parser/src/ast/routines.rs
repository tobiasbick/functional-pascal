use super::Visibility;
use super::{Decl, FormalParam, Stmt, TypeExpr};
use fpas_lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    /// Generic type parameters: `<T>`, `<A, B>`.
    pub type_params: Vec<String>,
    pub params: Vec<FormalParam>,
    pub return_type: TypeExpr,
    pub body: FuncBody,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureDecl {
    pub name: String,
    /// Generic type parameters: `<T>`, `<A, B>`.
    pub type_params: Vec<String>,
    pub params: Vec<FormalParam>,
    pub body: FuncBody,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncBody {
    Forward,
    Block { nested: Vec<Decl>, stmts: Vec<Stmt> },
}
