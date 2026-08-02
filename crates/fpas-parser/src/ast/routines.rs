use super::Visibility;
use super::{Decl, FormalParam, Stmt, TypeExpr};
use fpas_lexer::Span;

/// Parsed named function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    /// Declared function name.
    pub name: String,
    /// Generic type parameters in declaration order.
    pub type_params: Vec<super::TypeParam>,
    /// Formal parameters in declaration order.
    pub params: Vec<FormalParam>,
    /// Declared function result type.
    pub return_type: TypeExpr,
    /// Parsed function body.
    pub body: FuncBody,
    /// Visibility specified for the declaration.
    pub visibility: Visibility,
    /// Source span of the complete declaration.
    pub span: Span,
}

/// Parsed named procedure declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureDecl {
    /// Declared procedure name.
    pub name: String,
    /// Generic type parameters in declaration order.
    pub type_params: Vec<super::TypeParam>,
    /// Formal parameters in declaration order.
    pub params: Vec<FormalParam>,
    /// Parsed procedure body.
    pub body: FuncBody,
    /// Visibility specified for the declaration.
    pub visibility: Visibility,
    /// Source span of the complete declaration.
    pub span: Span,
}

/// Callable body shape.
///
/// **Documentation:** `docs/pascal/language/functions/README.md` (from the repository root).
#[derive(Debug, Clone, PartialEq)]
pub enum FuncBody {
    /// Block body containing nested declarations followed by statements.
    Block {
        /// Declarations local to the callable.
        nested: Vec<Decl>,
        /// Statements in execution order.
        stmts: Vec<Stmt>,
    },
}
