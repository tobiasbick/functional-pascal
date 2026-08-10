//! Pure runtime value operations shared by bytecode execution and debugger evaluation.

mod comparison;
mod index;
mod scalar;

use fpas_bytecode::Value;
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_DIVISION_BY_ZERO,
    RUNTIME_MODULO_BY_ZERO, RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

/// Broad error class for one side-effect-free value operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueOperationErrorKind {
    /// Runtime values are incompatible with the requested operation.
    Type,
    /// Runtime values have valid types but violate an operation domain or bound.
    Domain,
}

/// Stable failure returned before any caller-owned state is modified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueOperationError {
    pub(crate) kind: ValueOperationErrorKind,
    pub(crate) code: DiagnosticCode,
    pub(crate) message: String,
    pub(crate) hint: String,
}

impl ValueOperationError {
    pub(crate) fn type_mismatch(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            kind: ValueOperationErrorKind::Type,
            code: RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub(crate) fn domain(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::domain_with_code(RUNTIME_NUMERIC_DOMAIN_ERROR, message, hint)
    }

    pub(crate) fn division_by_zero(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::domain_with_code(RUNTIME_DIVISION_BY_ZERO, message, hint)
    }

    pub(crate) fn modulo_by_zero(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::domain_with_code(RUNTIME_MODULO_BY_ZERO, message, hint)
    }

    pub(crate) fn array_bounds(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::domain_with_code(RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, message, hint)
    }

    pub(crate) fn missing_dictionary_key(
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self::domain_with_code(RUNTIME_DICT_KEY_NOT_FOUND, message, hint)
    }

    fn domain_with_code(
        code: DiagnosticCode,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            kind: ValueOperationErrorKind::Domain,
            code,
            message: message.into(),
            hint: hint.into(),
        }
    }
}

/// Unary operation supported by the read-only runtime-value boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnaryOperation {
    Negate,
    Not,
}

/// Binary operation supported by the read-only runtime-value boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    RealDivide,
    IntegerDivide,
    Modulo,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    In,
}

pub(crate) fn unary(
    operation: UnaryOperation,
    value: &Value,
) -> Result<Value, ValueOperationError> {
    scalar::unary(operation, value)
}

pub(crate) fn binary(
    operation: BinaryOperation,
    left: &Value,
    right: &Value,
) -> Result<Value, ValueOperationError> {
    match operation {
        BinaryOperation::Equal
        | BinaryOperation::NotEqual
        | BinaryOperation::Less
        | BinaryOperation::LessEqual
        | BinaryOperation::Greater
        | BinaryOperation::GreaterEqual
        | BinaryOperation::In => comparison::binary(operation, left, right),
        _ => scalar::binary(operation, left, right),
    }
}

pub(crate) fn field(value: &Value, name: &str) -> Result<Value, ValueOperationError> {
    index::field(value, name)
}

pub(crate) fn index(value: &Value, key: &Value) -> Result<Value, ValueOperationError> {
    index::index(value, key)
}
