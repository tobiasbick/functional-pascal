use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_parser::{Expr, LocalBinding, TypeExpr, VarDef};

impl Checker {
    /// Checks an initializer and preserves its type and static capture guarantees.
    pub(crate) fn check_var_def(&mut self, v: &VarDef, mutable: bool) {
        self.check_binding(
            &v.name,
            Some(&v.type_expr),
            &v.value,
            v.span,
            mutable,
            false,
        );
    }

    /// Checks a statement-local binding and exposes its checked type to editor queries.
    ///
    /// **Documentation:** `docs/pascal/language/basics/local-variables.md`
    pub(crate) fn check_local_binding(&mut self, binding: &LocalBinding, mutable: bool) {
        let ty = self.check_binding(
            &binding.name,
            binding.type_expr.as_ref(),
            &binding.value,
            binding.span,
            mutable,
            !mutable,
        );
        if binding.using {
            let resolved = self.resolve_visible_type(&ty);
            let release_name = match &resolved {
                crate::types::Ty::Record(record) => {
                    crate::types::native_resource_release(&record.name)
                        .map(str::to_owned)
                        .or_else(|| {
                            record.release_method.as_ref().map(|method| {
                                let name = if record.name.contains('.') {
                                    record.name.clone()
                                } else if let Some(owner) = &record.owner_unit {
                                    format!("{owner}.{}", record.name)
                                } else {
                                    record.name.clone()
                                };
                                format!("{name}.{method}")
                            })
                        })
                }
                _ => None,
            };
            if let Some(name) = release_name {
                let signature = if let crate::types::Ty::Record(record) = &resolved {
                    record
                        .release_method
                        .as_ref()
                        .and_then(|release| {
                            record
                                .methods
                                .iter()
                                .find(|(method, _)| method.eq_ignore_ascii_case(release))
                        })
                        .map(|(_, method)| match method {
                            crate::types::MethodKind::Procedure(procedure) => {
                                crate::types::Ty::Procedure(procedure.clone())
                            }
                            crate::types::MethodKind::Function(function) => {
                                crate::types::Ty::Function(function.clone())
                            }
                        })
                } else {
                    None
                }
                .or_else(|| {
                    self.scopes
                        .imports
                        .lookup_metadata(&name)
                        .or_else(|| self.scopes.lookup(&name))
                        .or_else(|| {
                            name.rsplit_once('.')
                                .and_then(|(unit, _)| {
                                    self.scopes.imports.source_name_for(unit, &name)
                                })
                                .and_then(|source| self.scopes.imports.lookup_source(&source))
                        })
                        .map(|symbol| symbol.ty.clone())
                });
                if let Some(signature) = signature {
                    let native = matches!(&resolved, crate::types::Ty::Record(record) if crate::types::native_resource_release(&record.name).is_some())
                        && name != "Std.Net.Http.CloseStream";
                    let custom = matches!(&resolved, crate::types::Ty::Record(record) if record.release_method.is_some());
                    if custom
                        && !matches!(&signature, crate::types::Ty::Procedure(procedure)
                        if procedure.params.len() == 1
                            && procedure.type_params.is_empty()
                            && procedure.params[0].mode == crate::types::ParamMode::Value
                            && resolved.assignment_compatible_with(&self.resolve_visible_type(&procedure.params[0].ty)))
                    {
                        self.error_with_code(
                            fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                            format!("Release operation `{name}` has an invalid signature"),
                            "A release procedure must take exactly its record as a value receiver.",
                            binding.span,
                        );
                        return;
                    }
                    self.resource_releases.insert(
                        (binding.span.source_id, binding.span.offset),
                        crate::ResourceRelease {
                            name,
                            signature,
                            native,
                        },
                    );
                } else {
                    self.error_with_code(
                        fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                        format!("Release operation `{name}` is unavailable"),
                        "Import the unit providing the resource release operation.",
                        binding.span,
                    );
                }
            } else if ty != crate::types::Ty::Error {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_TYPE_MISMATCH,
                    format!("Type `{ty}` has no resource release contract"),
                    "Use a resource type or declare a release procedure on the record.",
                    binding.span,
                );
            }
        }
        if binding.type_expr.is_none() {
            self.binding_types
                .insert((binding.span.source_id, binding.span.offset), ty);
        }
    }

    /// Checks a binding while retaining expression identity for capture metadata.
    pub(crate) fn check_binding(
        &mut self,
        name: &str,
        type_expr: Option<&TypeExpr>,
        value: &Expr,
        span: fpas_lexer::Span,
        mutable: bool,
        constant: bool,
    ) -> crate::types::Ty {
        if self.check_import_alias_collision(name, span) {
            return crate::types::Ty::Error;
        }
        let declared_ty = if let Some(type_expr) = type_expr {
            let declared = self.resolve_type_expr(type_expr);
            let actual = self.check_expr_with_expected(value, Some(&declared));
            let context = if constant {
                "const initializer"
            } else {
                "variable initializer"
            };
            self.check_type_compat(&declared, &actual, context, span);
            declared
        } else {
            self.infer_local_binding(name, value, span, mutable)
        };

        let stored_ty = declared_ty;

        let capability = if mutable {
            self.assignment_value_capability(value)
        } else {
            self.expr_capability(Self::expr_lookup_key(value))
        };
        let constant_info = constant.then(|| self.binding_constant_info(value, &stored_ty));
        let symbol = Symbol {
            constant: constant_info,
            ty: stored_ty.clone(),
            mutable,
            kind: if constant {
                SymbolKind::Const
            } else {
                SymbolKind::Var
            },
            capability,
        };
        let installed = if self
            .predeclared_bindings
            .remove(&(span.source_id, span.offset))
        {
            if let Some(existing) = self.scopes.lookup_mut(name) {
                *existing = symbol;
            }
            true
        } else {
            self.scopes.define_with_declaration(name, symbol, span)
        };
        if installed {
            self.record_callable_result_binding(name, value, mutable);
        }
        if !installed {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!(
                    "Duplicate {} `{name}`",
                    if constant { "constant" } else { "variable" }
                ),
                "Each name must be unique in the same scope.",
                span,
            );
        }
        self.record_pending_capture_binding(name, Some(value));
        stored_ty
    }

    /// Classify static inputs without evaluating or reordering their runtime initializers.
    pub(crate) fn binding_constant_info(
        &mut self,
        value: &Expr,
        ty: &crate::types::Ty,
    ) -> crate::scope::ConstantInfo {
        let compile_time = self.const_initializer_is_compile_time_known(value, ty);
        crate::scope::ConstantInfo {
            record: compile_time
                .then(|| self.record_constant_value(value))
                .flatten(),
            compile_time,
            value: if compile_time {
                self.scalar_constant_value(value)
            } else {
                None
            },
        }
    }
}
