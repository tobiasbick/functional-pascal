//! Typed operator kinds and operand/result categories.
//!
//! Documentation: `docs/pascal/language/basics/operators.md`.

/// A typed binary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    /// Integer addition.
    AddInteger,
    /// Integer subtraction.
    SubtractInteger,
    /// Integer multiplication.
    MultiplyInteger,
    /// Integer division.
    DivideInteger,
    /// Integer remainder.
    RemainderInteger,
    /// Real addition.
    AddReal,
    /// Real subtraction.
    SubtractReal,
    /// Real multiplication.
    MultiplyReal,
    /// Real division.
    DivideReal,
    /// A dynamically checked generic numeric addition.
    AddDynamic,
    /// A dynamically checked generic numeric subtraction.
    SubtractDynamic,
    /// A dynamically checked generic numeric multiplication.
    MultiplyDynamic,
    /// A dynamically checked generic numeric division.
    DivideDynamic,
    /// Equality comparison of like-typed values.
    Equal,
    /// Inequality comparison of like-typed values.
    NotEqual,
    /// Integer less-than comparison.
    LessThanInteger,
    /// Integer greater-than comparison.
    GreaterThanInteger,
    /// Integer less-than-or-equal comparison.
    LessEqualInteger,
    /// Integer greater-than-or-equal comparison.
    GreaterEqualInteger,
    /// Real less-than comparison.
    LessThanReal,
    /// Real greater-than comparison.
    GreaterThanReal,
    /// Real less-than-or-equal comparison.
    LessEqualReal,
    /// Real greater-than-or-equal comparison.
    GreaterEqualReal,
    /// String less-than comparison.
    LessThanString,
    /// String greater-than comparison.
    GreaterThanString,
    /// String less-than-or-equal comparison.
    LessEqualString,
    /// String greater-than-or-equal comparison.
    GreaterEqualString,
    /// Dynamically checked less-than comparison.
    LessThanDynamic,
    /// Dynamically checked greater-than comparison.
    GreaterThanDynamic,
    /// Dynamically checked less-than-or-equal comparison.
    LessEqualDynamic,
    /// Dynamically checked greater-than-or-equal comparison.
    GreaterEqualDynamic,
    /// Boolean conjunction.
    AndBoolean,
    /// Boolean disjunction.
    OrBoolean,
    /// UTF-8 string concatenation.
    ConcatString,
}

/// A typed unary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    /// Checked integer negation.
    NegateInteger,
    /// IEEE-754 real negation.
    NegateReal,
    /// Dynamically checked generic numeric negation.
    NegateDynamic,
    /// Boolean negation.
    NotBoolean,
    /// Convert an integer value to a real value.
    IntegerToReal,
}

/// Returns the required operand and result categories for a binary operation.
#[must_use]
pub const fn binary_categories(operation: BinaryOperation) -> (TypeCategory, TypeCategory) {
    match operation {
        BinaryOperation::AddInteger
        | BinaryOperation::SubtractInteger
        | BinaryOperation::MultiplyInteger
        | BinaryOperation::DivideInteger
        | BinaryOperation::RemainderInteger => (TypeCategory::Integer, TypeCategory::Integer),
        BinaryOperation::AddReal
        | BinaryOperation::SubtractReal
        | BinaryOperation::MultiplyReal
        | BinaryOperation::DivideReal => (TypeCategory::Real, TypeCategory::Real),
        BinaryOperation::AddDynamic
        | BinaryOperation::SubtractDynamic
        | BinaryOperation::MultiplyDynamic
        | BinaryOperation::DivideDynamic => (TypeCategory::Dynamic, TypeCategory::Dynamic),
        BinaryOperation::Equal | BinaryOperation::NotEqual => {
            (TypeCategory::Same, TypeCategory::Boolean)
        }
        BinaryOperation::LessThanInteger
        | BinaryOperation::GreaterThanInteger
        | BinaryOperation::LessEqualInteger
        | BinaryOperation::GreaterEqualInteger => (TypeCategory::Integer, TypeCategory::Boolean),
        BinaryOperation::LessThanReal
        | BinaryOperation::GreaterThanReal
        | BinaryOperation::LessEqualReal
        | BinaryOperation::GreaterEqualReal => (TypeCategory::Real, TypeCategory::Boolean),
        BinaryOperation::LessThanString
        | BinaryOperation::GreaterThanString
        | BinaryOperation::LessEqualString
        | BinaryOperation::GreaterEqualString => (TypeCategory::String, TypeCategory::Boolean),
        BinaryOperation::LessThanDynamic
        | BinaryOperation::GreaterThanDynamic
        | BinaryOperation::LessEqualDynamic
        | BinaryOperation::GreaterEqualDynamic => (TypeCategory::Comparable, TypeCategory::Boolean),
        BinaryOperation::AndBoolean | BinaryOperation::OrBoolean => {
            (TypeCategory::Boolean, TypeCategory::Boolean)
        }
        BinaryOperation::ConcatString => (TypeCategory::String, TypeCategory::String),
    }
}

/// Returns the required operand and result categories for a unary operation.
#[must_use]
pub const fn unary_categories(operation: UnaryOperation) -> (TypeCategory, TypeCategory) {
    match operation {
        UnaryOperation::NegateInteger => (TypeCategory::Integer, TypeCategory::Integer),
        UnaryOperation::NegateReal => (TypeCategory::Real, TypeCategory::Real),
        UnaryOperation::NegateDynamic => (TypeCategory::Dynamic, TypeCategory::Dynamic),
        UnaryOperation::NotBoolean => (TypeCategory::Boolean, TypeCategory::Boolean),
        UnaryOperation::IntegerToReal => (TypeCategory::Integer, TypeCategory::Real),
    }
}

/// A compact type category used when validating typed operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCategory {
    /// Requires matching operand types.
    Same,
    /// Requires the lowered Unit type.
    Unit,
    /// Requires the lowered boolean type.
    Boolean,
    /// Requires the lowered integer type.
    Integer,
    /// Requires the lowered real type.
    Real,
    /// Requires the lowered string type.
    String,
    /// Requires the lowered dynamic type.
    Dynamic,
    /// Requires a scalar comparable or dynamically erased type.
    Comparable,
}
