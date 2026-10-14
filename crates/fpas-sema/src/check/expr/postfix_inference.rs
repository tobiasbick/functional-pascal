//! Complete unresolved native receivers from supplied arguments and result context.
//! See `docs/pascal/language/functions/generic-routines.md`.

use super::super::Checker;
use crate::std_registry::native_operation_by_implementation;
use crate::types::{Ty, merge_inferred_types, needs_expected_type};
use fpas_parser::PostfixOperation;
use std::collections::{HashMap, HashSet};

impl Checker {
    /// Propagates catalog evidence back to an incomplete primary expression.
    pub(super) fn infer_postfix_base_context(
        &mut self,
        operations: &[PostfixOperation],
        types: &[Ty],
        expected: Option<&Ty>,
    ) -> Option<Ty> {
        let base = types.first()?;
        if !needs_expected_type(base) {
            return None;
        }
        let mut context = expected
            .cloned()
            .unwrap_or_else(|| types.last().unwrap().clone());
        for (index, operation) in operations.iter().enumerate().rev() {
            let input = &types[index];
            let output = &types[index + 1];
            if output.compatible_with(&context) {
                context = merge_inferred_types(output, &context);
            } else {
                context = output.clone();
            }
            context = match operation {
                PostfixOperation::Index { .. } if matches!(input, Ty::Array(_)) => {
                    Ty::Array(Box::new(context))
                }
                PostfixOperation::Field { name, .. } => {
                    let Ty::Record(record) = input else {
                        return None;
                    };
                    let mut record = record.as_ref().clone();
                    let (_, field) = record
                        .fields
                        .iter_mut()
                        .find(|(field, _)| field.eq_ignore_ascii_case(name))?;
                    *field = merge_inferred_types(field, &context);
                    Ty::Record(std::sync::Arc::new(record))
                }
                PostfixOperation::MethodCall { args, span, .. } => {
                    let call = self
                        .fluent_calls
                        .get(&Self::postfix_operation_lookup_key(operation))?;
                    let native = native_operation_by_implementation(&call.name)?;
                    let receiver = match input {
                        Ty::Array(_) => Ty::Array(Box::new(Ty::GenericParam("T".into(), None))),
                        Ty::Option(_) => Ty::Option(Box::new(Ty::GenericParam("T".into(), None))),
                        Ty::Dict(_, _) => Ty::Dict(
                            Box::new(Ty::GenericParam("K".into(), None)),
                            Box::new(Ty::GenericParam("V".into(), None)),
                        ),
                        Ty::Result(_, _) => Ty::Result(
                            Box::new(Ty::GenericParam("T".into(), None)),
                            Box::new(Ty::GenericParam("E".into(), None)),
                        ),
                        _ => return None,
                    };
                    let signature = (native.signature)();
                    let mut bindings = ["t", "k", "v", "e"]
                        .into_iter()
                        .map(|name| (name.to_owned(), Ty::Error))
                        .collect::<HashMap<_, _>>();
                    self.collect_type_param_bindings(
                        &receiver,
                        input,
                        &mut bindings,
                        &mut HashSet::new(),
                        *span,
                    );
                    for (position, arg) in args.iter().enumerate() {
                        let parameter = match arg.argument_name() {
                            Some(name) => signature
                                .params
                                .iter()
                                .find(|param| param.name.eq_ignore_ascii_case(name)),
                            None => signature.params.get(position),
                        }?;
                        let actual = self
                            .expr_types
                            .get(&Self::expr_lookup_key(arg.argument_value()))
                            .cloned()?;
                        self.collect_type_param_bindings(
                            &parameter.ty,
                            &actual,
                            &mut bindings,
                            &mut HashSet::new(),
                            *span,
                        );
                    }
                    self.collect_type_param_bindings(
                        &signature.return_type,
                        &context,
                        &mut bindings,
                        &mut HashSet::new(),
                        *span,
                    );
                    Self::substitute_type_params(&receiver, &bindings)
                }
                _ => return None,
            };
            context = merge_inferred_types(input, &context);
        }
        (context != *base && !context.is_error()).then_some(context)
    }
}
