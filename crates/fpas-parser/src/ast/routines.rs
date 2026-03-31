use super::Visibility;
use super::{Decl, FormalParam, Stmt, TypeExpr};
use fpas_lexer::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    /// Generic type parameters: `<T>`, `<T: Comparable>`, `<A, B>`.
    pub type_params: Vec<super::TypeParam>,
    pub params: Vec<FormalParam>,
    pub return_type: TypeExpr,
    pub body: FuncBody,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureDecl {
    pub name: String,
    /// Generic type parameters: `<T>`, `<T: Comparable>`, `<A, B>`.
    pub type_params: Vec<super::TypeParam>,
    pub params: Vec<FormalParam>,
    pub body: FuncBody,
    pub visibility: Visibility,
    pub span: Span,
}

/// Callable body shape.
///
/// **Documentation:** `docs/pascal/04-functions.md` (from the repository root).
#[derive(Debug, Clone, PartialEq)]
pub enum FuncBody {
    Block { nested: Vec<Decl>, stmts: Vec<Stmt> },
}
