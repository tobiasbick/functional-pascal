//! Protocol-neutral expression IR, limits, and result records.

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
}

impl Default for DebugEvaluationLimits {
    fn default() -> Self {
        Self {
            max_expression_bytes: 4_096,
            max_depth: 64,
            max_operations: 1_024,
            max_traversals: 16,
            max_output_bytes: 65_536,
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
