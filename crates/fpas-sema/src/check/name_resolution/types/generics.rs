use super::super::Checker;
use crate::types::{
    EnumTy, EnumVariantTy, FunctionTy, GenericParamDef, ParamTy, ProcedureTy, RecordTy, Ty,
};
use fpas_diagnostics::codes::{SEMA_CONSTRAINT_VIOLATION, SEMA_UNKNOWN_TYPE};
use fpas_lexer::Span;

impl Checker {
    /// Apply type arguments to a generic type. For non-generic types with type
    /// args, this is currently a pass-through (type erasure: the VM doesn't
    /// need concrete type args at runtime).
    ///
    /// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
    pub(crate) fn apply_type_args(&mut self, base: Ty, args: &[Ty], span: Span) -> Ty {
        match &base {
            Ty::Record(record) if !record.type_params.is_empty() => {
                self.instantiate_record_type(record, args, span)
            }
            Ty::Enum(enum_ty) if !enum_ty.type_params.is_empty() => {
                self.instantiate_enum_type(enum_ty, args, span)
            }
            _ => base,
        }
    }

    fn instantiate_record_type(&mut self, record: &RecordTy, args: &[Ty], span: Span) -> Ty {
        if !self.validate_type_arg_count(&record.name, record.type_params.len(), args.len(), span) {
            return Ty::Error;
        }
        self.validate_constraints(&record.type_params, args, span);

        let mapping = Self::type_arg_mapping(&record.type_params, args);
        let fields = record
            .fields
            .iter()
            .map(|(name, ty)| (name.clone(), Self::substitute_type_params(ty, &mapping)))
            .collect();
        let methods = record
            .methods
            .iter()
            .map(|(name, method)| (name.clone(), method.clone()))
            .collect();

        Ty::Record(RecordTy {
            name: record.name.clone(),
            type_params: Vec::new(),
            fields,
            methods,
            implements: record.implements.clone(),
        })
    }

    fn instantiate_enum_type(&mut self, enum_ty: &EnumTy, args: &[Ty], span: Span) -> Ty {
        if !self.validate_type_arg_count(&enum_ty.name, enum_ty.type_params.len(), args.len(), span)
        {
            return Ty::Error;
        }
        self.validate_constraints(&enum_ty.type_params, args, span);

        let mapping = Self::type_arg_mapping(&enum_ty.type_params, args);
        let variants = enum_ty
            .variants
            .iter()
            .map(|variant| EnumVariantTy {
                name: variant.name.clone(),
                fields: variant
                    .fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), Self::substitute_type_params(ty, &mapping)))
                    .collect(),
            })
            .collect();

        Ty::Enum(EnumTy {
            name: enum_ty.name.clone(),
            type_params: Vec::new(),
            variants,
        })
    }

    /// Check that each concrete type argument satisfies its parameter's constraint.
    pub(crate) fn validate_constraints(
        &mut self,
        type_params: &[GenericParamDef],
        args: &[Ty],
        span: Span,
    ) {
        for (param, arg) in type_params.iter().zip(args.iter()) {
            // Skip validation for error types and unresolved generic params.
            if arg.is_error() || matches!(arg, Ty::GenericParam(..)) {
                continue;
            }
            if let Some(constraint) = param.constraint
                && !constraint.satisfied_by(arg)
            {
                self.error_with_code(
                    SEMA_CONSTRAINT_VIOLATION,
                    format!(
                        "Type `{arg}` does not satisfy constraint `{}` on parameter `{}`",
                        constraint.display_name(),
                        param.name,
                    ),
                    format!(
                        "The `{}` constraint requires a type that supports {}.",
                        constraint.display_name(),
                        match constraint {
                            crate::types::TypeConstraint::Comparable =>
                                "comparison operators (=, <>, <, >, <=, >=)",
                            crate::types::TypeConstraint::Numeric =>
                                "arithmetic operators (+, -, *, /, div, mod)",
                            crate::types::TypeConstraint::Printable => "string conversion",
                        },
                    ),
                    span,
                );
            }
        }
    }

    fn validate_type_arg_count(
        &mut self,
        type_name: &str,
        expected: usize,
        actual: usize,
        span: Span,
    ) -> bool {
        if expected == actual {
            return true;
        }

        self.error_with_code(
            SEMA_UNKNOWN_TYPE,
            format!("`{type_name}` expects {expected} type argument(s), got {actual}"),
            format!("Provide exactly {expected} type argument(s)."),
            span,
        );
        false
    }

    fn type_arg_mapping<'a>(
        type_params: &'a [GenericParamDef],
        args: &'a [Ty],
    ) -> Vec<(&'a str, &'a Ty)> {
        type_params
            .iter()
            .zip(args.iter())
            .map(|(param, arg)| (param.name.as_str(), arg))
            .collect()
    }

    /// Replace `GenericParam("T")` with concrete types based on the mapping.
    fn substitute_type_params(ty: &Ty, mapping: &[(&str, &Ty)]) -> Ty {
        match ty {
            Ty::GenericParam(name, _) | Ty::Named(name) => {
                Self::lookup_type_param(name, mapping).unwrap_or_else(|| ty.clone())
            }
            Ty::Array(inner) => Ty::Array(Box::new(Self::substitute_type_params(inner, mapping))),
            Ty::Result(ok, err) => Ty::Result(
                Box::new(Self::substitute_type_params(ok, mapping)),
                Box::new(Self::substitute_type_params(err, mapping)),
            ),
            Ty::Option(inner) => Ty::Option(Box::new(Self::substitute_type_params(inner, mapping))),
            Ty::Channel(inner) => {
                Ty::Channel(Box::new(Self::substitute_type_params(inner, mapping)))
            }
            Ty::Task(inner) => Ty::Task(Box::new(Self::substitute_type_params(inner, mapping))),
            Ty::Ref(inner) => Ty::Ref(Box::new(Self::substitute_type_params(inner, mapping))),
            Ty::Dict(k, v) => Ty::Dict(
                Box::new(Self::substitute_type_params(k, mapping)),
                Box::new(Self::substitute_type_params(v, mapping)),
            ),
            Ty::Function(f) => Ty::Function(FunctionTy {
                type_params: f.type_params.clone(),
                params: f
                    .params
                    .iter()
                    .map(|p| ParamTy {
                        mutable: p.mutable,
                        name: p.name.clone(),
                        ty: Self::substitute_type_params(&p.ty, mapping),
                    })
                    .collect(),
                return_type: Box::new(Self::substitute_type_params(&f.return_type, mapping)),
            }),
            Ty::Procedure(p) => Ty::Procedure(ProcedureTy {
                type_params: p.type_params.clone(),
                variadic: p.variadic,
                params: p
                    .params
                    .iter()
                    .map(|param| ParamTy {
                        mutable: param.mutable,
                        name: param.name.clone(),
                        ty: Self::substitute_type_params(&param.ty, mapping),
                    })
                    .collect(),
            }),
            _ => ty.clone(),
        }
    }

    fn lookup_type_param(name: &str, mapping: &[(&str, &Ty)]) -> Option<Ty> {
        mapping.iter().find_map(|(param_name, concrete)| {
            name.eq_ignore_ascii_case(param_name)
                .then(|| (*concrete).clone())
        })
    }
}
