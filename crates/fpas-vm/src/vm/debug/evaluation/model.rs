//! Protocol-neutral expression IR, limits, and result records.

use std::time::Duration;

/// Maximum resources consumed by one stopped-state expression evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugEvaluationLimits {
    /// Largest accepted UTF-8 expression source in bytes.
    pub max_expression_bytes: usize,
    /// Largest accepted validated IR nesting depth.
    pub max_depth: usize,
    /// Largest number of evaluated IR nodes.
    pub max_operations: usize,
    /// Largest number of aggregate field or index traversals.
    pub max_traversals: usize,
    /// Largest rendered result in UTF-8 bytes.
    pub max_output_bytes: usize,
    /// Largest number of controlled calls in one expression.
    pub max_calls: usize,
    /// Largest nested controlled-call depth.
    pub max_call_depth: usize,
    /// Largest instruction count dispatched by one expression's sandbox calls.
    pub max_call_instructions: u64,
    /// Largest number of values copied into the detached sandbox graph.
    pub max_detached_values: usize,
    /// Wall-clock limit shared by all sandbox calls in one expression.
    pub call_timeout: Duration,
}

impl Default for DebugEvaluationLimits {
    fn default() -> Self {
        Self {
            max_expression_bytes: 4_096,
            max_depth: 64,
            max_operations: 1_024,
            max_traversals: 16,
            max_output_bytes: 65_536,
            max_calls: 64,
            max_call_depth: 32,
            max_call_instructions: 1_000_000,
            max_detached_values: 65_536,
            call_timeout: Duration::from_secs(2),
        }
    }
}

/// Side-effect-free debugger expression accepted by the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum DebugExpression {
    /// Signed 64-bit integer literal.
    Integer(i64),
    /// IEEE-754 real literal.
    Real(f64),
    /// Boolean literal.
    Boolean(bool),
    /// UTF-8 string literal.
    String(String),
    /// Visible source binding resolved case-insensitively.
    Name(String),
    /// Exact compiler-visible callable name, resolved only at invocation.
    Callable(String),
    /// Unary runtime-value operation.
    Unary {
        /// Requested operator.
        operation: DebugUnaryOperation,
        /// Operand expression.
        operand: Box<Self>,
    },
    /// Binary runtime-value operation.
    Binary {
        /// Requested operator.
        operation: DebugBinaryOperation,
        /// Left operand.
        left: Box<Self>,
        /// Right operand.
        right: Box<Self>,
    },
    /// Stored record or enum field read.
    Field {
        /// Aggregate expression.
        base: Box<Self>,
        /// Stored field name.
        name: String,
    },
    /// Read-only array, dictionary, or string index.
    Index {
        /// Aggregate expression.
        base: Box<Self>,
        /// Index or dictionary key expression.
        index: Box<Self>,
    },
    /// Controlled invocation of a named or first-class callable expression.
    Call {
        /// Named callable or expression yielding a function value.
        callee: Box<Self>,
        /// Arguments in source order.
        arguments: Vec<Self>,
    },
    /// Controlled instance method invocation.
    MethodCall {
        /// Receiver value.
        receiver: Box<Self>,
        /// Exact source member name.
        name: String,
        /// Explicit arguments in source order.
        arguments: Vec<Self>,
    },
    /// Side-effect-free array construction.
    Array(Vec<Self>),
    /// Side-effect-free dictionary construction.
    Dictionary(Vec<(Self, Self)>),
    /// Record construction inferred from an exact executable layout field set.
    Record(Vec<(String, Self)>),
    /// Copy-on-write record update in the detached result graph.
    RecordUpdate {
        /// Existing record expression.
        base: Box<Self>,
        /// Replacement fields.
        fields: Vec<(String, Self)>,
    },
    /// Construct `Result.Ok`.
    ResultOk(Box<Self>),
    /// Construct `Result.Error`.
    ResultError(Box<Self>),
    /// Construct `Option.Some`.
    OptionSome(Box<Self>),
    /// Construct `Option.None`.
    OptionNone,
    /// Unwrap `Result.Ok` or `Option.Some` without propagation outside the expression.
    Try(Box<Self>),
}

/// Runtime-resolved controlled call target supplied to the sandbox boundary.
#[derive(Debug, Clone)]
pub(in crate::vm::debug) enum DebugCallTarget {
    /// Exact executable or intrinsic name.
    Named(String),
    /// First-class callable value.
    Value(fpas_bytecode::Value),
    /// Exact member on a runtime record receiver.
    Method {
        /// Receiver passed as the implicit first argument.
        receiver: fpas_bytecode::Value,
        /// Source member name.
        name: String,
    },
    /// Property getter fallback after no stored field matched.
    Property {
        /// Receiver passed to the getter.
        receiver: fpas_bytecode::Value,
        /// Source property name.
        name: String,
    },
    /// Construct a record whose layout exactly matches these field names.
    Record {
        /// Field names in source order.
        fields: Vec<String>,
    },
}

/// Unary operator in [`DebugExpression`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugUnaryOperation {
    /// Numeric negation with unary `-`.
    Negate,
    /// Boolean negation with `not`.
    Not,
}

/// Binary operator in [`DebugExpression`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugBinaryOperation {
    /// `+`.
    Add,
    /// `-`.
    Subtract,
    /// `*`.
    Multiply,
    /// `/`.
    RealDivide,
    /// `div`.
    IntegerDivide,
    /// `mod`.
    Modulo,
    /// `and`.
    And,
    /// `or`.
    Or,
    /// `xor`.
    Xor,
    /// `shl`.
    ShiftLeft,
    /// `shr`.
    ShiftRight,
    /// `=`.
    Equal,
    /// `<>`.
    NotEqual,
    /// `<`.
    Less,
    /// `<=`.
    LessEqual,
    /// `>`.
    Greater,
    /// `>=`.
    GreaterEqual,
    /// `in`.
    In,
}

/// Rendered result of one debugger expression evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugEvaluateResult {
    /// Bounded FPAS value summary.
    pub value: String,
    /// Runtime or source type name.
    pub type_name: String,
    /// Stop-local reference for aggregate expansion, or zero for a leaf.
    pub variables_reference: u64,
    /// Number of named children.
    pub named_variables: usize,
    /// Number of indexed children.
    pub indexed_variables: usize,
}
