//! Routine entry state and storage for captured parameters.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::*;
use fpas_ir::LocalId;

impl LoweringContext {
    /// Build routine entry storage and cell-backed capture owners.
    pub(in crate::lowering) fn new(input: FunctionInput<'_>) -> Result<Self, CompileError> {
        let FunctionInput {
            name,
            source_name,
            id,
            result,
            parameters: parameter_types,
            captures,
            globals,
            constants,
            metadata,
            callables,
            closure_targets,
            intrinsic_task_targets,
            cell_names,
            type_table,
        } = input;
        let parameters = parameter_types
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                ValueId::try_from_index(index)
                    .map(|id| ValueDefinition {
                        id,
                        ty: parameter.ty,
                    })
                    .map_err(|error| {
                        internal_compiler_error(
                            error.to_string(),
                            "Split the routine into smaller functions.",
                            1,
                            1,
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut locals = Vec::with_capacity(parameter_types.len() + captures.len());
        let mut bindings = Vec::with_capacity(parameter_types.len() + captures.len());
        let mut debug = fpas_ir::FunctionDebugInfo {
            scopes: vec![fpas_ir::DebugScope {
                id: 0,
                parent: None,
            }],
            ..fpas_ir::FunctionDebugInfo::default()
        };
        let mut entry = empty_block(BlockId::new(0));
        for (input, parameter) in parameter_types.iter().zip(&parameters) {
            let referenced = match type_table.kind(input.ty) {
                Some(fpas_ir::IrType::Reference(inner)) => Some(*inner),
                _ => None,
            };
            let logical_ty = referenced.unwrap_or(input.ty);
            let local = LocalId::try_from_index(locals.len()).map_err(|error| {
                internal_compiler_error(
                    error.to_string(),
                    "Reduce the number of routine parameters.",
                    1,
                    1,
                )
            })?;
            locals.push(Local {
                id: local,
                ty: input.ty,
                mutable: true,
                capture: None,
            });
            debug.bindings.push(fpas_ir::DebugBinding {
                local,
                name: input.name.clone(),
                kind: fpas_ir::DebugBindingKind::Parameter,
                ty: logical_ty,
                mutable: referenced.is_some(),
                scope: 0,
                declaration: input.declaration,
                hidden: false,
                cell_backed: false,
                initializer: None,
            });
            bindings.push(Binding {
                name: input.name.to_ascii_lowercase(),
                storage: if referenced.is_some() {
                    BindingStorage::Reference(local)
                } else {
                    BindingStorage::Local(local)
                },
                ty: logical_ty,
                depth: 0,
                cell: false,
            });
            if referenced.is_some() || !cell_names.contains(&input.name.to_ascii_lowercase()) {
                entry.instructions.push(Instruction {
                    source: None,
                    result: None,
                    operation: Operation::WriteLocal {
                        value: parameter.id,
                        local,
                    },
                });
            }
        }
        for (index, capture) in captures.iter().enumerate() {
            let local = LocalId::try_from_index(parameter_types.len().saturating_add(index))
                .map_err(|error| {
                    internal_compiler_error(
                        error.to_string(),
                        "Reduce the number of captured values in this routine.",
                        1,
                        1,
                    )
                })?;
            locals.push(Local {
                id: local,
                ty: capture.storage_ty,
                mutable: capture.kind != fpas_ir::CaptureKind::Value,
                capture: Some(capture.kind),
            });
            debug.bindings.push(fpas_ir::DebugBinding {
                local,
                name: capture.name.clone(),
                kind: fpas_ir::DebugBindingKind::Capture,
                ty: capture.ty,
                mutable: capture.kind != fpas_ir::CaptureKind::Value,
                scope: 0,
                declaration: capture.declaration,
                hidden: false,
                cell_backed: capture.kind != fpas_ir::CaptureKind::Value,
                initializer: None,
            });
            bindings.push(Binding {
                name: capture.name.to_ascii_lowercase(),
                storage: BindingStorage::Local(local),
                ty: capture.ty,
                depth: 0,
                cell: capture.kind != fpas_ir::CaptureKind::Value,
            });
        }
        let mut context = Self {
            program_name: name.to_ascii_lowercase(),
            source_name: source_name.to_string(),
            function_id: id,
            result_type: result,
            parameters,
            captures: captures
                .iter()
                .map(|capture| fpas_ir::CaptureDeclaration {
                    ty: capture.ty,
                    kind: capture.kind,
                })
                .collect(),
            callables,
            closure_targets,
            intrinsic_task_targets,
            cell_names,
            globals,
            constants,
            import_aliases: metadata.import_aliases.clone(),
            type_table,
            expr_types: metadata.expr_types.clone(),
            intrinsic_calls: metadata.intrinsic_calls.clone(),
            record_defaults: metadata.record_defaults.clone(),
            record_constructions: metadata.record_constructions.clone(),
            projection_types: metadata.projection_types.clone(),
            pattern_infos: metadata.pattern_infos.clone(),
            exhaustive_cases: metadata.exhaustive_cases.clone(),
            value_calls: metadata.value_calls.clone(),
            blocks: vec![entry],
            current: BlockId::new(0),
            locals,
            bindings,
            loops: Vec::new(),
            scope_depth: 0,
            debug,
            debug_scope: 0,
            debug_scope_stack: Vec::new(),
            next_value: fpas_ir::checked_count("parameter count", parameter_types.len()).map_err(
                |error| {
                    internal_compiler_error(
                        error.to_string(),
                        "Split the routine into smaller functions.",
                        1,
                        1,
                    )
                },
            )?,
            max_call_arguments: 0,
            can_spawn_tasks: false,
        };
        context.initialize_parameter_cells(parameter_types)?;
        Ok(context)
    }

    fn initialize_parameter_cells(
        &mut self,
        inputs: &[ParameterInput],
    ) -> Result<(), CompileError> {
        for (index, input) in inputs.iter().enumerate() {
            if !self.is_cell_backed(&input.name)
                || matches!(
                    self.type_kind(input.ty),
                    Some(fpas_ir::IrType::Reference(_))
                )
            {
                continue;
            }
            let span = input.declaration.map(Span::from).unwrap_or(Span {
                offset: 0,
                length: 0,
                line: 1,
                column: 1,
                source_id: 0,
            });
            let storage_ty = self.cell_type(input.ty, span)?;
            let cell = self.emit_value(
                Operation::MakeCell(self.parameters[index].id),
                storage_ty,
                span,
            )?;
            let local = self.locals[index].id;
            self.locals[index].ty = storage_ty;
            self.mark_binding_cell(&input.name, input.ty);
            self.initialize_local(local, cell, span)?;
        }
        Ok(())
    }
}
