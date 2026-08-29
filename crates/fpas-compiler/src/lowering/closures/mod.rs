//! Anonymous-closure discovery, capture typing, and body lowering.

mod bound_methods;
mod discover;

use std::collections::{BTreeMap, BTreeSet, HashMap};

use fpas_ir::{CaptureKind, Function, FunctionId};
use fpas_parser::{Expr, FuncBody};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::context::{
    BoundMethodTarget, Callable, CaptureInput, ClosureTarget, FunctionInput, LoweringContext,
    ParameterInput, unsupported,
};
use super::types;

pub(super) struct ClosureRoutine<'a> {
    pub(super) expression: &'a Expr,
    pub id: FunctionId,
    pub(super) name: String,
    pub(super) captures: Vec<CaptureInput>,
    pub owner: FunctionId,
}

pub(super) struct BoundMethodRoutine {
    pub id: FunctionId,
    name: String,
    target: FunctionId,
    receiver_ty: fpas_ir::TypeId,
    parameters: Vec<fpas_ir::TypeId>,
    result: fpas_ir::TypeId,
    span: fpas_lexer::Span,
    pub owner: FunctionId,
}

pub(super) struct ClosureRegistry<'a> {
    pub routines: Vec<ClosureRoutine<'a>>,
    pub targets: HashMap<usize, ClosureTarget>,
    pub bound_routines: Vec<BoundMethodRoutine>,
    pub bound_targets: HashMap<usize, BoundMethodTarget>,
    pub cell_names: HashMap<FunctionId, BTreeSet<String>>,
    callables: BTreeMap<String, Callable>,
    source_name: String,
    pub(super) next_id: u32,
}

impl<'a> ClosureRegistry<'a> {
    pub fn new(first_id: u32, callables: BTreeMap<String, Callable>, source_name: &str) -> Self {
        Self {
            routines: Vec::new(),
            targets: HashMap::new(),
            bound_routines: Vec::new(),
            bound_targets: HashMap::new(),
            cell_names: HashMap::new(),
            callables,
            source_name: source_name.to_string(),
            next_id: first_id,
        }
    }

    /// Mark owner locals that named nested routines capture as cells.
    ///
    /// Anonymous closures already record this during discovery. Named nested captures
    /// come from the callable table and must use the same MakeCell lowering.
    pub fn seed_named_nested_cells(
        &mut self,
        owners: &[FunctionId],
        runtime_names: &[String],
        callables: &BTreeMap<String, Callable>,
    ) {
        for (index, owner) in owners.iter().copied().enumerate() {
            let Some(name) = runtime_names.get(index) else {
                continue;
            };
            let Some(callable) = callables.get(&name.to_ascii_lowercase()) else {
                continue;
            };
            for capture in &callable.captures {
                if capture.kind == CaptureKind::Value {
                    continue;
                }
                self.cell_names
                    .entry(owner)
                    .or_default()
                    .insert(capture.name.to_ascii_lowercase());
            }
        }
    }

    pub fn lower(
        &self,
        routine: &ClosureRoutine<'a>,
        metadata: &AnalysisMetadata,
        callables: &BTreeMap<String, Callable>,
        types: &mut types::TypeTable,
        globals: &BTreeMap<String, super::context::GlobalBinding>,
        constants: &BTreeMap<String, fpas_ir::Constant>,
    ) -> Result<(Function, types::TypeTable), CompileError> {
        let Expr::Closure(closure) = routine.expression else {
            return Err(unsupported(
                routine.expression.span(),
                "closure registry entry",
            ));
        };
        let parameters = closure
            .params
            .iter()
            .map(|parameter| {
                types
                    .type_expr(&parameter.type_expr)
                    .map(|ty| ParameterInput {
                        name: parameter.name.clone(),
                        ty,
                        declaration: Some(parameter.span.diagnostic_span_or_synthetic()),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result = closure
            .return_type
            .as_ref()
            .map(|result| types.type_expr(result))
            .transpose()?
            .unwrap_or(types::UNIT);
        let mut context = LoweringContext::new(FunctionInput {
            name: &routine.name,
            source_name: &self.source_name,
            id: routine.id,
            result,
            parameters: &parameters,
            captures: &routine.captures,
            globals: globals.clone(),
            constants: constants.clone(),
            metadata,
            callables: callables.clone(),
            closure_targets: self.targets.clone(),
            bound_method_targets: self.bound_targets.clone(),
            cell_names: self
                .cell_names
                .get(&routine.id)
                .cloned()
                .unwrap_or_default(),
            type_table: types.clone(),
        })?;
        let FuncBody::Block { stmts, .. } = &closure.body;
        context.lower_statements(stmts)?;
        context.finish(closure.span)
    }
}
