//! Record constructors share joint field inference with enum variants.
//! See `docs/pascal/language/types/generics.md`.

use super::super::construction_inference::ConstructorField;
use crate::{
    check::Checker,
    types::{RecordTy, Ty},
};
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::sync::Arc;

impl Checker {
    /// Instantiate a constructor using every explicit field and its expected type.
    pub(super) fn infer_record_construction(
        &mut self,
        record: &Arc<RecordTy>,
        fields: &[(&str, &Expr, Span)],
        expected: Option<&Ty>,
        span: Span,
    ) -> Arc<RecordTy> {
        if record.type_params.is_empty() {
            return record.clone();
        }
        let context = expected.map(|ty| self.resolve_visible_type(ty));
        let context = match &context {
            Some(Ty::Record(context)) if context.name.eq_ignore_ascii_case(&record.name) => {
                Some(context)
            }
            _ => None,
        };
        let fixed = if record.type_args.is_empty() {
            context.map_or(&record.type_args, |context| &context.type_args)
        } else {
            &record.type_args
        };
        let supplied: Vec<_> = fields
            .iter()
            .filter_map(|&(name, value, _)| {
                let (_, declared) = record
                    .fields
                    .iter()
                    .find(|(field, _)| field.eq_ignore_ascii_case(name))?;
                Some(ConstructorField {
                    value,
                    declared,
                    expected: context
                        .and_then(|context| {
                            context
                                .fields
                                .iter()
                                .find(|(field, _)| field.eq_ignore_ascii_case(name))
                        })
                        .map(|(_, ty)| ty),
                })
            })
            .collect();
        let arguments = self.infer_constructor_arguments(
            &record.name,
            &record.type_params,
            fixed,
            &supplied,
            span,
        );
        let template = self
            .scopes
            .lookup_type(&record.name)
            .and_then(|symbol| match &symbol.ty {
                Ty::Record(template) => Some(template.clone()),
                _ => None,
            })
            .unwrap_or_else(|| record.clone());
        Arc::new(template.instantiate(arguments))
    }
}
