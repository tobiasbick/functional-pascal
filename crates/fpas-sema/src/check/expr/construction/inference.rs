//! Shared contextual arguments and completion for nominal data constructors.

use std::collections::HashMap;

use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;

use crate::check::Checker;
use crate::types::{Ty, TypeArguments};

impl Checker {
    /// Provide concrete payload context without leaking unresolved declaration parameters.
    pub(super) fn construction_payload_context(
        &self,
        template: &Ty,
        declared: &Ty,
        inferred: &TypeArguments,
    ) -> Ty {
        let mut context = template
            .type_parameters()
            .iter()
            .map(|parameter| (parameter.identity.clone(), Ty::Error))
            .collect::<HashMap<_, _>>();
        context.extend(inferred.clone());
        declared.substitute(&context)
    }

    /// Seed construction arguments when context names the same nominal declaration.
    pub(super) fn construction_bindings(
        &self,
        template: &Ty,
        expected: Option<&Ty>,
    ) -> TypeArguments {
        let arguments = match (template, expected.map(|ty| self.resolve_visible_type(ty))) {
            (Ty::Record(template), Some(Ty::Record(expected)))
                if template.name.eq_ignore_ascii_case(&expected.name) =>
            {
                expected.type_args.clone()
            }
            (Ty::Enum(template), Some(Ty::Enum(expected)))
                if template.name.eq_ignore_ascii_case(&expected.name) =>
            {
                expected.type_args.clone()
            }
            _ => Vec::new(),
        };
        template
            .type_parameters()
            .iter()
            .zip(arguments)
            .filter(|(_, argument)| !argument.has_inference_holes())
            .map(|(parameter, argument)| (parameter.identity.clone(), argument))
            .collect()
    }

    /// Require complete inferred arguments and enforce declaration constraints.
    pub(super) fn finish_data_construction(
        &mut self,
        template: &Ty,
        inferred: &TypeArguments,
        source_name: &str,
        span: Span,
    ) -> Ty {
        let parameters = template.type_parameters();
        let arguments = parameters
            .iter()
            .map(|parameter| {
                inferred
                    .get(&parameter.identity)
                    .cloned()
                    .unwrap_or(Ty::Error)
            })
            .collect::<Vec<_>>();
        if arguments.iter().any(Ty::has_inference_holes) {
            if self.inference_depth > 0 {
                return template.instantiate(&arguments).unwrap_or(Ty::Error);
            }
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Cannot infer every type argument for `{source_name}`"),
                "Add an explicit type annotation that supplies the constructor's type arguments.",
                span,
            );
            return Ty::Error;
        }
        self.validate_constraints(parameters, &arguments, span);
        if arguments.is_empty() {
            template.clone()
        } else {
            template.instantiate(&arguments).unwrap_or(Ty::Error)
        }
    }
}
