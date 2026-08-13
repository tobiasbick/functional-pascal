//! Stopped-state discovery and atomic construction of complete wrapper variants.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::Arc;
use std::sync::atomic::Ordering;

use fpas_bytecode::Value;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::{
    DebugAssignmentSelector, DebugAssignmentTarget, DebugVariantConstructionResult,
    DebugVariantDescription, VariantMetadata, WrapperMetadata, complete_value,
    constructible_description, ordered_field_expressions, require_constructible_fields,
    require_wrapper, unknown_variant,
};

impl DebugSession {
    /// Describe constructible variants for one textual mutable wrapper target.
    ///
    /// Discovery is read-only: it does not increment the inspection generation or mutate live state.
    ///
    /// # Errors
    ///
    /// Returns a stable state, frame, target, mutability, type, or metadata error.
    pub fn describe_variant(
        &self,
        target: &DebugAssignmentTarget,
        frame_id: Option<u64>,
    ) -> Result<DebugVariantDescription, DebugSessionError> {
        self.describe_variant_with_limits(target, frame_id, DebugEvaluationLimits::default())
    }

    /// Describe constructible variants using explicit selector-evaluation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::describe_variant`].
    pub fn describe_variant_with_limits(
        &self,
        assignment: &DebugAssignmentTarget,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugVariantDescription, DebugSessionError> {
        self.require_stopped("variant.describe")?;
        let result = self
            .resolve_wrapper_target(assignment, frame_id, limits)
            .and_then(|resolved| {
                constructible_description(self.executable.executable(), &resolved.wrapper)
            });
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    /// Construct one complete variant and commit it onto a mutable wrapper target.
    ///
    /// # Errors
    ///
    /// Returns a stable state, frame, target, variant, field-set, expression, type, resource, or
    /// availability error. Failure leaves live program state and inspection handles unchanged.
    pub fn construct_variant(
        &mut self,
        target: &DebugAssignmentTarget,
        variant: &str,
        fields: &[(String, DebugExpression)],
        frame_id: Option<u64>,
    ) -> Result<DebugVariantConstructionResult, DebugSessionError> {
        self.construct_variant_with_limits(
            target,
            variant,
            fields,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Construct one complete variant using explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::construct_variant`].
    pub fn construct_variant_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        variant: &str,
        fields: &[(String, DebugExpression)],
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugVariantConstructionResult, DebugSessionError> {
        self.require_stopped("variant.construct")?;
        let result = (|| {
            let resolved = self.resolve_variant_construction(
                assignment,
                PreparedFields { variant, fields },
                frame_id,
                limits,
            )?;
            let replacement = complete_value(
                &self.executable,
                &resolved.variant,
                resolved.field_values,
                limits,
            )?;
            let value =
                self.commit_mutation(resolved.task_id, &resolved.target, replacement, limits)?;
            Ok(DebugVariantConstructionResult {
                variant: resolved.variant.canonical_name,
                value,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn resolve_wrapper_target(
        &self,
        assignment: &DebugAssignmentTarget,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<ResolvedWrapperTarget, DebugSessionError> {
        let (_, root_target, current) = self.named_target(assignment, frame_id)?;
        let target = match current {
            None => root_target,
            Some(current) => {
                let indexes = index_expressions(assignment);
                let values = if indexes.is_empty() {
                    Vec::new()
                } else {
                    self.evaluate_runtime_values(&indexes, frame_id, limits)?
                };
                super::super::mutation::target_with_value(
                    self.executable.executable(),
                    assignment,
                    root_target,
                    current,
                    &values,
                )?
                .0
            }
        };
        let wrapper = require_wrapper(self.executable.executable(), target.expected_type)?;
        Ok(ResolvedWrapperTarget { wrapper })
    }

    fn resolve_variant_construction(
        &self,
        assignment: &DebugAssignmentTarget,
        construction: PreparedFields<'_>,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<ResolvedVariantConstruction, DebugSessionError> {
        let (task_id, root_target, current) = self.named_target(assignment, frame_id)?;
        let resolved = match current {
            None => {
                let wrapper =
                    require_wrapper(self.executable.executable(), root_target.expected_type)?;
                let prepared =
                    prepare_construction(self.executable.executable(), &wrapper, construction)?;
                let field_values =
                    self.evaluate_runtime_values(&prepared.expressions, frame_id, limits)?;
                ResolvedVariantConstruction {
                    task_id,
                    target: root_target,
                    variant: prepared.variant,
                    field_values,
                }
            }
            Some(current) => {
                let executable = Arc::clone(&self.executable);
                let indexes = index_expressions(assignment);
                let (prepared, field_values) = self.evaluate_runtime_values_with_dynamic_suffix(
                    &indexes,
                    frame_id,
                    limits,
                    move |values| {
                        let (target, _) = super::super::mutation::target_with_value(
                            executable.executable(),
                            assignment,
                            root_target,
                            current,
                            values,
                        )?;
                        let wrapper =
                            require_wrapper(executable.executable(), target.expected_type)?;
                        let prepared =
                            prepare_construction(executable.executable(), &wrapper, construction)?;
                        let expressions = prepared.expressions;
                        Ok(((target, prepared.variant), expressions))
                    },
                )?;
                ResolvedVariantConstruction {
                    task_id,
                    target: prepared.0,
                    variant: prepared.1,
                    field_values,
                }
            }
        };
        validate_field_values(&self.executable, &resolved, limits)?;
        Ok(resolved)
    }

    fn named_target(
        &self,
        assignment: &DebugAssignmentTarget,
        frame_id: Option<u64>,
    ) -> Result<NamedTarget, DebugSessionError> {
        let task_id = self.task_for_frame(frame_id)?;
        let (root_target, current) = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?
            .resolve_named_mutation_target(frame_id, &assignment.root)?;
        if current.is_none() && !assignment.selectors.is_empty() {
            return Err(uninitialized_root_path(&assignment.root));
        }
        Ok((task_id, root_target, current))
    }
}

type NamedTarget = (
    u64,
    crate::vm::debug::inspection::MutationTarget,
    Option<Value>,
);

fn index_expressions(assignment: &DebugAssignmentTarget) -> Vec<DebugExpression> {
    assignment
        .selectors
        .iter()
        .filter_map(|selector| match selector {
            DebugAssignmentSelector::Field(_) => None,
            DebugAssignmentSelector::Index(expression) => Some(expression.clone()),
        })
        .collect()
}

fn prepare_construction(
    executable: &fpas_bytecode::Executable,
    wrapper: &WrapperMetadata,
    construction: PreparedFields<'_>,
) -> Result<PreparedConstruction, DebugSessionError> {
    let selected = wrapper
        .find_canonical(construction.variant)
        .map_err(|_| unknown_variant(construction.variant, wrapper))?
        .clone();
    require_constructible_fields(executable, &selected)?;
    let ordered = ordered_field_expressions(&selected, construction.fields)?;
    Ok(PreparedConstruction {
        expressions: ordered.into_iter().cloned().collect(),
        variant: selected,
    })
}

fn validate_field_values(
    executable: &fpas_bytecode::VerifiedExecutable,
    resolved: &ResolvedVariantConstruction,
    limits: DebugEvaluationLimits,
) -> Result<(), DebugSessionError> {
    for (field, value) in resolved.variant.fields.iter().zip(&resolved.field_values) {
        super::super::mutation::validate_value(executable, field.ty, value, limits.max_depth)?;
    }
    Ok(())
}

struct PreparedFields<'a> {
    variant: &'a str,
    fields: &'a [(String, DebugExpression)],
}

struct PreparedConstruction {
    variant: VariantMetadata,
    expressions: Vec<DebugExpression>,
}

struct ResolvedWrapperTarget {
    wrapper: WrapperMetadata,
}

struct ResolvedVariantConstruction {
    task_id: u64,
    target: crate::vm::debug::inspection::MutationTarget,
    variant: VariantMetadata,
    field_values: Vec<Value>,
}

fn uninitialized_root_path(name: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariablePathUnsupported,
        message: format!(
            "debug variable target `{name}` has no writable descendants before initialization"
        ),
        hint: "Initialize the complete binding before editing fields, indexes, or payload descendants."
            .to_string(),
    }
}
