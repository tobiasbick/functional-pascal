use super::{Expr, FunctionDecl, ProcedureDecl, TypeExpr};
use fpas_lexer::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    Public,
    #[default]
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
/// Used on function and procedure headings: `function Foo<T>(x: T): T`.
///
/// **Documentation:** `docs/pascal/language/functions/generic-routines.md`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    /// Optional constraint name: `Comparable`, `Numeric`, `Printable`.
    pub constraint: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    pub name: String,
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
    /// Computed properties backed by instance accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub properties: Vec<RecordProperty>,
    /// Event members backed by `Option of Handler` accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub events: Vec<RecordEvent>,
    pub span: Span,
}

/// A computed property declared inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/record-properties.md`
#[derive(Debug, Clone, PartialEq)]
pub struct RecordProperty {
    pub name: String,
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Instance function name after contextual `read`.
    pub read: Option<String>,
    /// Instance procedure name after contextual `write`.
    pub write: Option<String>,
    pub span: Span,
}

/// An event declared inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq)]
pub struct RecordEvent {
    pub name: String,
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Instance getter returning `Option of` the handler type.
    pub read: String,
    /// Instance setter accepting `Option of` the handler type.
    pub write: String,
    pub span: Span,
}

/// A function or procedure declared inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/record-methods.md`
#[derive(Debug, Clone, PartialEq)]
pub enum RecordMethod {
    /// Instance function: first parameter must be `Self`.
    Function(FunctionDecl),
    /// Static function: called through the type, no implicit receiver.
    StaticFunction(FunctionDecl),
    /// Static procedure: called through the type, no implicit receiver.
    StaticProcedure(ProcedureDecl),
    /// Instance procedure: first parameter must be `Self`.
    Procedure(ProcedureDecl),
}

impl RecordMethod {
    /// Return the visibility declared directly before this record routine.
    pub fn visibility(&self) -> Visibility {
        match self {
            Self::Function(function) | Self::StaticFunction(function) => function.visibility,
            Self::Procedure(procedure) | Self::StaticProcedure(procedure) => procedure.visibility,
        }
    }

    /// Return the source name of this record routine.
    pub fn name(&self) -> &str {
        match self {
            Self::Function(function) | Self::StaticFunction(function) => &function.name,
            Self::Procedure(procedure) | Self::StaticProcedure(procedure) => &procedure.name,
        }
    }
}

/// A field declaration inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/records.md`
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
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
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    pub fields: Vec<EnumMemberField>,
    pub span: Span,
}

/// A named, typed field inside an enum variant with associated data.
///
/// **Documentation:** `docs/pascal/language/types/enums.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumMemberField {
    pub name: String,
    pub type_expr: TypeExpr,
    pub span: Span,
}
