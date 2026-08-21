//! Lowering value, callable, capture, and loop descriptors.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{BlockId, FunctionId, GlobalId, LocalId, TypeId};
use fpas_sema::AnalysisMetadata;

use super::super::types;

#[derive(Debug, Clone)]
pub(super) struct Binding {
    pub name: String,
    pub storage: BindingStorage,
    pub ty: TypeId,
    pub depth: u32,
    pub cell: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BindingStorage {
    Local(LocalId),
}

#[derive(Debug, Clone)]
pub(crate) struct Callable {
    pub function: FunctionId,
    pub parameters: Vec<TypeId>,
    pub result: TypeId,
    pub value_type: TypeId,
    pub captures: Vec<CaptureInput>,
}

#[derive(Debug, Clone)]
/// One lexical capture and the source declaration that selected it.
pub(crate) struct CaptureInput {
    /// Source-level binding name.
    pub name: String,
    /// Value type exposed to the nested routine.
    pub ty: TypeId,
    /// Storage type used by the closure environment.
    pub storage_ty: TypeId,
    /// Capture representation used by the runtime.
    pub kind: fpas_ir::CaptureKind,
    /// Exact source declaration when the capture originates in user code.
    pub declaration: Option<fpas_ir::SourceSpan>,
}

#[derive(Debug, Clone)]
/// One lowered parameter and its optional source declaration identity.
pub(crate) struct ParameterInput {
    /// Source-level parameter name.
    pub name: String,
    /// Lowered parameter type.
    pub ty: TypeId,
    /// Exact source declaration when the parameter originates in user code.
    pub declaration: Option<fpas_ir::SourceSpan>,
}

#[derive(Debug, Clone)]
pub(crate) struct ClosureTarget {
    pub function: FunctionId,
    pub value_type: TypeId,
    pub captures: Vec<CaptureInput>,
}

#[derive(Debug, Clone)]
pub(crate) struct BoundMethodTarget {
    pub function: FunctionId,
    pub value_type: TypeId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopTargets {
    pub break_block: BlockId,
    pub continue_block: BlockId,
}

pub(crate) struct FunctionInput<'a> {
    pub name: &'a str,
    pub id: FunctionId,
    pub result: TypeId,
    pub parameters: &'a [ParameterInput],
    pub captures: &'a [CaptureInput],
    pub globals: BTreeMap<String, GlobalBinding>,
    pub constants: BTreeMap<String, fpas_ir::Constant>,
    pub metadata: &'a AnalysisMetadata,
    pub callables: BTreeMap<String, Callable>,
    pub closure_targets: HashMap<usize, ClosureTarget>,
    pub bound_method_targets: HashMap<usize, BoundMethodTarget>,
    pub cell_names: BTreeSet<String>,
    pub type_table: types::TypeTable,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlobalBinding {
    pub id: GlobalId,
    pub ty: TypeId,
}
