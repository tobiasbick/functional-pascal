//! Function, local, capture, and basic-block IR structures.

use crate::{BlockId, FunctionDebugInfo, FunctionId, LocalId, TypeId, ValueId};

/// A typed IR function in deterministic basic-block order.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    /// Stable function identifier.
    pub id: FunctionId,
    /// Canonical diagnostic name retained only as compiler metadata.
    pub name: String,
    /// Callable signature.
    pub signature: FunctionSignature,
    /// Ordered parameter values.
    pub parameters: Vec<ValueDefinition>,
    /// Ordered explicit FPAS locals.
    pub locals: Vec<Local>,
    /// Ordered closure capture declarations.
    pub captures: Vec<CaptureDeclaration>,
    /// Source-level debugger metadata kept outside operational instructions.
    pub debug: FunctionDebugInfo,
    /// Basic blocks in deterministic reverse-postorder.
    pub blocks: Vec<BasicBlock>,
    /// Function-local entry block.
    pub entry: BlockId,
    /// Largest argument count requested by any call in this function.
    pub max_call_arguments: u32,
    /// Whether this function may spawn tasks.
    pub can_spawn_tasks: bool,
}

impl Function {
    /// Returns a block by its typed identifier.
    #[must_use]
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.iter().find(|block| block.id == id)
    }

    /// Returns a local by its typed identifier.
    #[must_use]
    pub fn local(&self, id: LocalId) -> Option<&Local> {
        self.locals.iter().find(|local| local.id == id)
    }
}

/// A function's parameter and result types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    /// Ordered parameter types.
    pub parameters: Vec<TypeId>,
    /// Result type.
    pub result: TypeId,
}

/// A typed value definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueDefinition {
    /// Stable value identifier.
    pub id: ValueId,
    /// Lowered type of the value.
    pub ty: TypeId,
}

/// An explicit source-language local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    /// Stable local identifier.
    pub id: LocalId,
    /// Lowered type stored in the local.
    pub ty: TypeId,
    /// Whether source semantics allow assignment after initialization.
    pub mutable: bool,
    /// Capture representation when this local crosses a closure boundary.
    pub capture: Option<CaptureKind>,
}

/// A closure capture declaration in semantic capture order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureDeclaration {
    /// Lowered type captured by the function.
    pub ty: TypeId,
    /// Representation mandated by semantic capture analysis.
    pub kind: CaptureKind,
}

/// The representation of a captured value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureKind {
    /// The closure captures an immutable value.
    Value,
    /// The closure captures a mutable cell.
    Cell,
    /// The closure reuses an enclosing mutable cell.
    EnclosingCell,
}

/// A basic block with block parameters for incoming merge values.
#[derive(Debug, Clone, PartialEq)]
pub struct BasicBlock {
    /// Stable block identifier.
    pub id: BlockId,
    /// Merge values received from predecessor targets.
    pub parameters: Vec<BlockParameter>,
    /// Instructions evaluated in source-preserving order.
    pub instructions: Vec<crate::Instruction>,
    /// Exactly one terminator is required by validation.
    pub terminators: Vec<crate::Terminator>,
}

/// A typed parameter received by a basic block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockParameter {
    /// Stable value identifier for the merge value.
    pub id: ValueId,
    /// Lowered type of the merge value.
    pub ty: TypeId,
}
