use super::{BindingDef, Expr, FunctionDecl, ProcedureDecl, TypeExpr};
use fpas_lexer::Span;
use std::sync::Arc;

/// Visibility of a declaration or record member.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// The declaration is exported from its unit or record.
    Public,
    /// The declaration is visible only within its declaring scope.
    #[default]
    Private,
}

/// A parsed declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    /// An immutable binding definition.
    Const(BindingDef),
    /// A mutable binding definition.
    Var(BindingDef),
    /// A named type definition.
    TypeDef(TypeDef),
    /// A function declaration.
    Function(FunctionDecl),
    /// A procedure declaration.
    Procedure(ProcedureDecl),
}

impl Decl {
    /// Returns the visibility attached to the declaration.
    pub fn visibility(&self) -> Visibility {
        match self {
            Decl::Const(c) => c.visibility,
            Decl::Var(v) => v.visibility,
            Decl::TypeDef(td) => td.visibility,
            Decl::Function(f) => f.visibility,
            Decl::Procedure(p) => p.visibility,
        }
    }
}

/// A generic type parameter with optional constraint: `T` or `T: Comparable`.
///
/// Used on type and routine headings: `function Foo of (T)(Value: T): T`.
///
/// **Documentation:** `docs/pascal/language/functions/generic-routines.md`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    /// The type parameter name.
    pub name: String,
    /// Optional constraint name: `Comparable`, `Numeric`, `Printable`.
    pub constraint: Option<String>,
    /// Source position of the declared parameter name.
    pub span: Span,
}

/// A parsed named type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// The defined type name.
    pub name: String,
    /// Generic parameters in declaration order.
    pub type_params: Vec<TypeParam>,
    /// The type body assigned to the name.
    pub body: TypeBody,
    /// The declaration visibility.
    pub visibility: Visibility,
    /// The source span covering the definition.
    pub span: Span,
}

/// The body of a named type definition.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeBody {
    /// A record type definition.
    Record(RecordType),
    /// An enum type definition.
    Enum(EnumType),
    /// An alias of another type expression.
    Alias(TypeExpr),
}

/// A parsed record type body and its members.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    /// The record's stored fields.
    pub fields: Vec<FieldDef>,
    /// The source span covering the complete `record ... end` body.
    pub span: Span,
}

/// A field declaration inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/records.md`
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    /// The field name.
    pub name: String,
    /// The field type.
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Optional default expression used when the field is omitted from a record construction.
    /// Shared ownership preserves semantic node identities through default expansion.
    pub default_value: Option<Arc<Expr>>,
    /// The source span covering the field declaration.
    pub span: Span,
}

/// A parsed enum type body.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumType {
    /// The variants declared by the enum.
    pub members: Vec<EnumMember>,
    /// The source span covering the complete `enum ... end` body.
    pub span: Span,
}

/// A parsed enum variant, with either an integer value or associated data.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    /// The variant name.
    pub name: String,
    /// The explicitly assigned integer value, if present.
    pub value: Option<i64>,
    /// Associated-data fields. Empty for simple (valueless) variants.
    ///
    /// **Documentation:** `docs/pascal/language/types/enums.md`
    pub fields: Vec<EnumMemberField>,
    /// The source span covering the variant declaration.
    pub span: Span,
}

/// A named, typed field inside an enum variant with associated data.
///
/// **Documentation:** `docs/pascal/language/types/enums.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumMemberField {
    /// The associated-data field name.
    pub name: String,
    /// The associated-data field type.
    pub type_expr: TypeExpr,
    /// The source span covering the field declaration.
    pub span: Span,
}
