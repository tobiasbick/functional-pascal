use super::{Expr, FunctionDecl, ProcedureDecl, TypeExpr};
use fpas_lexer::Span;

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
    /// A constant definition.
    Const(ConstDef),
    /// An immutable variable definition.
    Var(VarDef),
    /// A mutable variable definition.
    MutableVar(VarDef),
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
            Decl::Var(v) | Decl::MutableVar(v) => v.visibility,
            Decl::TypeDef(td) => td.visibility,
            Decl::Function(f) => f.visibility,
            Decl::Procedure(p) => p.visibility,
        }
    }
}

/// A parsed constant definition with an explicit type and initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstDef {
    /// The declared constant name.
    pub name: String,
    /// The declared type.
    pub type_expr: TypeExpr,
    /// The initializer expression.
    pub value: Expr,
    /// The declaration visibility.
    pub visibility: Visibility,
    /// The source span covering the definition.
    pub span: Span,
}

/// A parsed variable definition with an explicit type and initializer.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDef {
    /// The declared variable name.
    pub name: String,
    /// The declared type.
    pub type_expr: TypeExpr,
    /// The initializer expression.
    pub value: Expr,
    /// The declaration visibility.
    pub visibility: Visibility,
    /// The source span covering the definition.
    pub span: Span,
}

/// A generic type parameter with optional constraint: `T` or `T: Comparable`.
///
/// Used on function and procedure headings: `function Foo<T>(x: T): T`.
///
/// **Documentation:** `docs/pascal/language/functions/generic-routines.md`
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    /// The type parameter name.
    pub name: String,
    /// Optional constraint name: `Comparable`, `Numeric`, `Printable`.
    pub constraint: Option<String>,
}

/// A parsed named type definition.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDef {
    /// The defined type name.
    pub name: String,
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
    /// The record's instance and static routines.
    pub methods: Vec<RecordMethod>,
    /// Computed properties backed by instance accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub properties: Vec<RecordProperty>,
    /// Event members backed by `Option of Handler` accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub events: Vec<RecordEvent>,
    /// The source span covering the complete `record ... end` body.
    pub span: Span,
}

/// A computed property declared inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/record-properties.md`
#[derive(Debug, Clone, PartialEq)]
pub struct RecordProperty {
    /// The property name.
    pub name: String,
    /// The value type exposed by the property.
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Instance function name after contextual `read`.
    pub read: Option<String>,
    /// Instance procedure name after contextual `write`.
    pub write: Option<String>,
    /// The source span covering the property declaration.
    pub span: Span,
}

/// An event declared inside a `record … end` block.
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq)]
pub struct RecordEvent {
    /// The event name.
    pub name: String,
    /// The handler type exposed by the event.
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Instance getter returning `Option of` the handler type.
    pub read: String,
    /// Instance setter accepting `Option of` the handler type.
    pub write: String,
    /// The source span covering the event declaration.
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
    /// The field name.
    pub name: String,
    /// The field type.
    pub type_expr: TypeExpr,
    /// Member visibility; private when no modifier was written.
    pub visibility: Visibility,
    /// Optional default expression used when the field is omitted from a record literal.
    /// Only valid on a named record type definition, not on anonymous literals.
    pub default_value: Option<Expr>,
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
