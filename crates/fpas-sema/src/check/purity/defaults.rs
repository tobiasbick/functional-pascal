//! Generic capability obligations follow the defaults actually selected at construction.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use crate::{
    check::Checker,
    types::{GenericParameterId, RecordTy, Ty},
};
use fpas_lexer::Span;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

type DefaultKey = (String, String);

/// Requirements inferred from checked operations, keyed by declaring field identity.
#[derive(Default)]
pub(in crate::check) struct DefaultPurity {
    requirements: HashMap<DefaultKey, Vec<GenericParameterId>>,
    active: Option<DefaultFrame>,
    uses: Vec<DefaultUse>,
}

/// The generic record parameters whose proofs may be consumed by one default.
pub(in crate::check) struct DefaultFrame {
    key: DefaultKey,
    parameters: Vec<GenericParameterId>,
    required: RefCell<Vec<GenericParameterId>>,
}

#[derive(Clone)]
struct DefaultUse {
    key: DefaultKey,
    arguments: HashMap<GenericParameterId, Ty>,
    owner: Option<(DefaultKey, Vec<GenericParameterId>)>,
    proofs: HashSet<GenericParameterId>,
    span: Span,
}

impl Checker {
    /// Export the nominal arguments required by this field's checked operations.
    pub(crate) fn default_type_requirements(
        &self,
        record: &str,
        field: &str,
    ) -> Vec<GenericParameterId> {
        self.default_purity
            .requirements
            .get(&(record.to_ascii_lowercase(), field.to_ascii_lowercase()))
            .cloned()
            .unwrap_or_default()
    }

    /// Restore requirements without importing implementation AST nodes.
    pub(crate) fn install_default_type_requirements(
        &mut self,
        record: &str,
        field: &str,
        parameters: &[GenericParameterId],
    ) {
        self.default_purity.requirements.insert(
            (record.to_ascii_lowercase(), field.to_ascii_lowercase()),
            parameters.to_vec(),
        );
    }
    /// Track which declaring generic parameter proofs the expression consumes.
    pub(in crate::check) fn begin_default_purity(
        &mut self,
        record: &str,
        field: &str,
        parameters: &[fpas_parser::TypeParam],
    ) -> Option<DefaultFrame> {
        let parameters = self
            .resolve_type_params(parameters)
            .into_iter()
            .map(|p| p.identity)
            .collect();
        self.default_purity.active.replace(DefaultFrame {
            key: (record.to_ascii_lowercase(), field.to_ascii_lowercase()),
            parameters,
            required: RefCell::new(Vec::new()),
        })
    }

    /// Store the checked requirements and restore an enclosing default context.
    pub(in crate::check) fn finish_default_purity(&mut self, previous: Option<DefaultFrame>) {
        if let Some(frame) = self.default_purity.active.take() {
            self.default_purity
                .requirements
                .insert(frame.key, frame.required.into_inner());
        }
        self.default_purity.active = previous;
    }

    /// Test each nominal proof independently using the shared component checker.
    pub(in crate::check) fn record_default_purity_requirement(&self, ty: &Ty, signatures: bool) {
        let Some(frame) = &self.default_purity.active else {
            return;
        };
        for parameter in &frame.parameters {
            if !self.default_capability(ty, &self.pure_parameters, Some(parameter), signatures) {
                let mut required = frame.required.borrow_mut();
                if !required.contains(parameter) {
                    required.push(parameter.clone());
                }
            }
        }
    }

    fn default_capability(
        &self,
        ty: &Ty,
        proofs: &HashSet<GenericParameterId>,
        disabled: Option<&GenericParameterId>,
        signatures: bool,
    ) -> bool {
        let resolve = |ty: &Ty| {
            let resolved = self.resolve_visible_type(ty);
            if let Ty::GenericParam(parameter) = &resolved {
                if Some(&parameter.identity) == disabled {
                    return Ty::Channel(Box::new(Ty::Integer));
                }
                if proofs.contains(&parameter.identity) {
                    return Ty::Integer;
                }
            }
            resolved
        };
        if signatures {
            ty.has_valid_pure_signatures_with(resolve)
        } else {
            ty.is_pure_data_with(resolve)
        }
    }

    /// Retain a selected default until forward declaration requirements are available.
    pub(in crate::check) fn record_default_use(
        &mut self,
        record: &RecordTy,
        field: &str,
        span: Span,
    ) {
        let arguments = record
            .type_params
            .iter()
            .zip(&record.type_args)
            .map(|(parameter, argument)| (parameter.identity.clone(), argument.clone()))
            .collect();
        self.default_purity.uses.push(DefaultUse {
            key: (record.name.to_ascii_lowercase(), field.to_ascii_lowercase()),
            arguments,
            owner: self
                .default_purity
                .active
                .as_ref()
                .map(|frame| (frame.key.clone(), frame.parameters.clone())),
            proofs: self.pure_parameters.clone(),
            span,
        });
    }

    /// Resolve default dependencies after every ordered declaration has been checked.
    pub(crate) fn validate_default_uses(&mut self) {
        let uses = std::mem::take(&mut self.default_purity.uses);
        loop {
            let mut changed = false;
            for usage in &uses {
                let Some((owner, parameters)) = &usage.owner else {
                    continue;
                };
                let required = self
                    .default_purity
                    .requirements
                    .get(&usage.key)
                    .cloned()
                    .unwrap_or_default();
                for argument in required
                    .iter()
                    .filter_map(|parameter| usage.arguments.get(parameter))
                {
                    for parameter in parameters {
                        if !self.default_capability(argument, &usage.proofs, Some(parameter), false)
                        {
                            let required = self
                                .default_purity
                                .requirements
                                .entry(owner.clone())
                                .or_default();
                            if !required.contains(parameter) {
                                required.push(parameter.clone());
                                changed = true;
                            }
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        for usage in uses {
            let required = self
                .default_purity
                .requirements
                .get(&usage.key)
                .cloned()
                .unwrap_or_default();
            for parameter in &required {
                if let Some(argument) = usage.arguments.get(parameter)
                    && !self.default_capability(argument, &usage.proofs, None, false)
                {
                    self.error_with_code(fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!("Default for `{}.{}` requires resource-free type arguments; `{argument}` does not satisfy its pure operations", usage.key.0, usage.key.1),
                        "Use resource-free data or explicitly supply this field so its default is not evaluated.", usage.span);
                }
            }
        }
    }
}
