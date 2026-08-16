//! Stopped-state seeded initialization of a descendant below empty storage.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::atomic::Ordering;

use super::*;
use crate::vm::debug::evaluation::{DebugEvaluationLimits, DebugExpression};
use crate::vm::debug::mutation::empty_storage::{
    DebugStorageInitializationResult, already_initialized, format_target, live_root_is_empty,
    rebuild_root, reject_identity_bearing, require_empty_root, resolve_existing_path,
    validate_seed,
};
use crate::vm::debug::mutation::{DebugAssignmentSelector, DebugAssignmentTarget};

impl DebugSession {
    /// Initialize one descendant below an empty mutable local or global from an explicit seed.
    ///
    /// # Errors
    ///
    /// Returns a stable state, target, eligibility, expression, type, path, or availability error.
    /// Failure leaves empty storage and the current inspection generation unchanged.
    pub fn initialize_storage(
        &mut self,
        target: &DebugAssignmentTarget,
        initializer: &DebugExpression,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugStorageInitializationResult, DebugSessionError> {
        self.initialize_storage_with_limits(
            target,
            initializer,
            expression,
            frame_id,
            DebugEvaluationLimits::default(),
        )
    }

    /// Initialize one empty-storage descendant using explicit evaluation and validation limits.
    ///
    /// # Errors
    ///
    /// Returns the same stable failures as [`Self::initialize_storage`].
    pub fn initialize_storage_with_limits(
        &mut self,
        assignment: &DebugAssignmentTarget,
        initializer: &DebugExpression,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugStorageInitializationResult, DebugSessionError> {
        self.require_stopped("storage.initialize")?;
        let result = (|| {
            let task_id = self.task_for_frame(frame_id)?;
            let (root_target, current) = self
                .inspections
                .get(&task_id)
                .ok_or_else(|| unknown_task(task_id))?
                .resolve_named_mutation_target(frame_id, &assignment.root)?;
            require_empty_root(assignment, &root_target)?;
            if current.is_some() {
                return Err(already_initialized(&assignment.root));
            }
            let index_expressions = assignment
                .selectors
                .iter()
                .filter_map(|selector| match selector {
                    DebugAssignmentSelector::Field(_) => None,
                    DebugAssignmentSelector::Index(expression) => Some(expression.clone()),
                })
                .collect::<Vec<_>>();
            let mut prefix = Vec::with_capacity(index_expressions.len().saturating_add(1));
            prefix.push(initializer.clone());
            prefix.extend(index_expressions);
            let executable = std::sync::Arc::clone(&self.executable);
            let root_type = root_target.expected_type;
            let root_for_resolve = root_target.clone();
            let (prepared, evaluated_replacement) = self.evaluate_runtime_values_with_checkpoint(
                &prefix,
                std::slice::from_ref(expression),
                frame_id,
                limits,
                move |values| {
                    let (seed, indexes) = values.split_first().ok_or_else(seed_unavailable)?;
                    validate_seed(&executable, root_type, seed, limits.max_depth)?;
                    let (descendant_target, _) = resolve_existing_path(
                        executable.executable(),
                        assignment,
                        root_for_resolve.clone(),
                        seed.clone(),
                        indexes,
                    )?;
                    Ok(PreparedSeed {
                        seed: seed.clone(),
                        indexes: indexes.to_vec(),
                        descendant_target,
                    })
                },
            )?;
            let replacement = evaluated_replacement
                .into_iter()
                .next()
                .ok_or_else(seed_unavailable)?;
            crate::vm::debug::mutation::validate_value(
                &self.executable,
                prepared.descendant_target.expected_type,
                &replacement,
                limits.max_depth,
            )?;
            reject_identity_bearing(&replacement, limits.max_depth)?;
            let rebuilt = rebuild_root(
                prepared.seed,
                &prepared.descendant_target.path,
                replacement.clone(),
            )?;
            validate_seed(&self.executable, root_type, &rebuilt, limits.max_depth)?;
            let mut commit_target = root_target;
            commit_target.path.clear();
            commit_target.expected_type = root_type;
            self.commit_initialized_root(CommitRequest {
                task_id,
                target: commit_target,
                rebuilt,
                descendant: replacement,
                assignment,
                indexes: &prepared.indexes,
                limits,
            })
        })();
        self.evaluation_cancelled.store(false, Ordering::Release);
        result
    }

    fn commit_initialized_root(
        &mut self,
        request: CommitRequest<'_>,
    ) -> Result<DebugStorageInitializationResult, DebugSessionError> {
        let CommitRequest {
            task_id,
            target,
            rebuilt,
            descendant,
            assignment,
            indexes,
            limits,
        } = request;
        crate::vm::debug::mutation::validate_replacement(
            &self.executable,
            &target,
            &rebuilt,
            limits.max_depth,
        )?;
        let inspection = self
            .inspections
            .get(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let generation = inspection.generation();
        if target.generation != generation {
            return Err(expired_target());
        }
        let root_value = inspection.evaluation_summary(&rebuilt, limits)?;
        let prepared_value = inspection.prepare_evaluation_result(&descendant, limits)?;
        let root = assignment.root.clone();
        let formatted_target = format_target(assignment, indexes);
        let worker = self
            .runtime
            .worker_mut(task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        if !live_root_is_empty(worker, &target)? {
            return Err(already_initialized(&root));
        }
        crate::vm::debug::mutation::commit(worker, generation, &target, rebuilt)?;
        if let Some(initializer) = target.initializer {
            worker.suppress_source_initializer(initializer);
        }
        self.invalidate_inspection();
        self.refresh_inspection();
        self.inspection_task_id = task_id;
        let inspection = self
            .inspections
            .get_mut(&task_id)
            .ok_or_else(|| unknown_task(task_id))?;
        let value = inspection.retain_prepared_evaluation_result(prepared_value);
        Ok(DebugStorageInitializationResult {
            root,
            target: formatted_target,
            root_value,
            value,
        })
    }
}

struct CommitRequest<'a> {
    task_id: u64,
    target: crate::vm::debug::inspection::MutationTarget,
    rebuilt: fpas_bytecode::Value,
    descendant: fpas_bytecode::Value,
    assignment: &'a DebugAssignmentTarget,
    indexes: &'a [fpas_bytecode::Value],
    limits: DebugEvaluationLimits,
}

struct PreparedSeed {
    seed: fpas_bytecode::Value,
    indexes: Vec<fpas_bytecode::Value>,
    descendant_target: crate::vm::debug::inspection::MutationTarget,
}

fn seed_unavailable() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableUnavailable,
        message: "debug storage initialization produced no seed or replacement value".to_string(),
        hint:
            "Retry at the current stop with one complete initializer and one replacement expression."
                .to_string(),
    }
}

fn expired_target() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableTargetExpired,
        message: "debug variable target belongs to an expired stop snapshot".to_string(),
        hint: "Request scopes and variables again for the current stop.".to_string(),
    }
}
