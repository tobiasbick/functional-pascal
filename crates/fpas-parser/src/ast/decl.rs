use super::{Expr, FunctionDecl, ProcedureDecl, TypeExpr};
use fpas_lexer::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    #[default]
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Const(ConstDef),
    Var(VarDef),
    MutableVar(VarDef),
    TypeDef(TypeDef),
    Function(FunctionDecl),
    Procedure(ProcedureDecl),
}

impl Decl {
    pub fn visibility(&self) -> Visibility {
        match self {
            Decl::Const(c) => c.visibility,
            Decl::Var(v) | Decl::MutableVar(v) => v.visibility,
            Decl::TypeDef(td) => td.visibility,
            Decl::Function(f) => f.visibility,
            Decl::Procedure(p) => p.visibility,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    pub name: String,
    pub type_expr: TypeExpr,
    pub value: Expr,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarDef {
    pub name: String,
    pub type_expr: TypeExpr,
    pub value: Expr,
    pub visibility: Visibility,
    pub span: Span,
}

/// A generic type parameter with optional constraint: `T` or `T: Comparable`.
///
/// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    /// Optional constraint name: `Comparable`, `Numeric`, `Printable`.
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
    /// Generic type parameters: `<T>`, `<T: Comparable>`, `<K, V>`.
    pub type_params: Vec<TypeParam>,
    pub body: TypeBody,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    Record(RecordType),
    Enum(EnumType),
    Alias(TypeExpr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    pub fields: Vec<FieldDef>,
    pub methods: Vec<RecordMethod>,
    pub span: Span,
}

/// A function or procedure declared inside a `record … end` block.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordMethod {
    Function(FunctionDecl),
    Procedure(ProcedureDecl),
}

/// A field declaration inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/05-types.md` (Record Types — Default Field Values)
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub type_expr: TypeExpr,
    /// Optional default expression used when the field is omitted from a record literal.
    /// Only valid on a named record type definition, not on anonymous literals.
    pub default_value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    pub members: Vec<EnumMember>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<i64>,
    /// Associated-data fields. Empty for simple (valueless) variants.
    ///
    /// **Documentation:** `docs/pascal/05-types.md`
    pub fields: Vec<EnumMemberField>,
    pub span: Span,
}

/// A named, typed field inside an enum variant with associated data.
///
/// **Documentation:** `docs/pascal/05-types.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumMemberField {
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}
