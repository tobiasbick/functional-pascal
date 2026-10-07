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
    /// Storage holds a reference to a caller variable (`var` parameter).
    pub reference: bool,
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
    /// The captured binding is a `var` parameter whose storage is a reference.
    pub reference: bool,
}

impl CaptureInput {
    /// Chooses the capture representation for one semantically analyzed capture.
    ///
    /// A `var` parameter is captured by value as its reference, so reads and writes in
    /// a non-escaping nested routine still reach the caller's variable.
    pub(crate) fn from_binding(
        capture: &fpas_sema::CaptureBinding,
        ty: TypeId,
        reuses_cell: bool,
        types: &mut types::TypeTable,
        span: fpas_lexer::Span,
    ) -> Result<Self, crate::CompileError> {
        let (kind, storage_ty) = if capture.reference {
            (fpas_ir::CaptureKind::Value, types.reference_type(ty, span)?)
        } else if reuses_cell {
            (
                fpas_ir::CaptureKind::EnclosingCell,
                types.cell_type(ty, span)?,
            )
        } else if capture.mutable {
            (fpas_ir::CaptureKind::Cell, types.cell_type(ty, span)?)
        } else {
            (fpas_ir::CaptureKind::Value, ty)
        };
        Ok(Self {
            name: capture.name.clone(),
            ty,
            storage_ty,
            kind,
            declaration: Some(capture.declaration.diagnostic_span_or_synthetic()),
            reference: capture.reference,
        })
    }
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
    /// `var` parameter: `ty` is a reference whose reads and writes reach the caller.
    pub reference: bool,
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
    /// Program or unit name that owns this function's source.
    pub source_name: &'a str,
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
    pub intrinsic_task_targets: HashMap<usize, BoundMethodTarget>,
    pub cell_names: BTreeSet<String>,
    pub type_table: types::TypeTable,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlobalBinding {
    pub id: GlobalId,
    pub ty: TypeId,
}
