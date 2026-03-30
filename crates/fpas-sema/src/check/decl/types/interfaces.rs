//! Semantic analysis for `interface … end` type declarations.
//!
//! **Documentation:** `docs/pascal/05-types.md` (Interfaces)

use super::Checker;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{FunctionTy, InterfaceTy, MethodKind, ParamTy, ProcedureTy, Ty};
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_UNKNOWN_TYPE};
use fpas_parser::{InterfaceType, RecordMethod, TypeDef};

impl Checker {
    /// Validate and register an `interface … end` type definition.
    ///
    /// The interface name is registered in the current scope as `Ty::Interface`. Its method
    /// signatures are resolved but not compiled — there is no body.
    pub(super) fn check_interface_type_def(&mut self, td: &TypeDef, iface: &InterfaceType) {
        // Register the name early so recursive interface references resolve.
        if !self.scopes.define(
            &td.name,
            Symbol {
                ty: Ty::Named(td.name.clone()),
                mutable: false,
                kind: SymbolKind::Type,
            },
        ) {
            self.error_with_code(
                SEMA_DUPLICATE_DECLARATION,
                format!("Duplicate type `{}`", td.name),
                "Each name must be unique in the same scope.",
                td.span,
            );
            return;
        }

        // Resolve the parent interface (extends clause).
        let extends = iface.extends.as_ref().map(|te| {
            let resolved = self.resolve_type_expr(te);
            match &resolved {
                Ty::Interface(parent) => parent.name.clone(),
                Ty::Named(name) => name.clone(),
                _ => {
                    self.error_with_code(
                        SEMA_UNKNOWN_TYPE,
                        format!("`{}` is not an interface", td.name),
                        "Only an interface name can appear after `extends`.",
                        iface.span,
                    );
                    "".to_string()
                }
            }
        });

        // Collect methods from the parent (if any), then add our own.
        let mut methods: Vec<(String, MethodKind)> = self
            .collect_inherited_methods(extends.as_deref())
            .unwrap_or_default();

        // Resolve method signatures declared on this interface.
        let iface_ty_placeholder = Ty::Named(td.name.clone());
        let own = self.with_type_params(&td.type_params, td.span, |checker| {
            checker.resolve_interface_method_sigs(
                &td.name,
                &iface_ty_placeholder,
                &iface.methods,
            )
        });
        methods.extend(own);

        let interface_ty = InterfaceTy {
            name: td.name.clone(),
            type_params: Self::resolve_type_params(&td.type_params),
            methods,
            extends,
        };
        let ty = Ty::Interface(interface_ty);

        if let Some(existing) = self.scopes.lookup_mut(&td.name) {
            *existing.ty_mut() = ty;
        }
    }

    /// Gather the method set of the parent interface (if named and resolvable).
    fn collect_inherited_methods(
        &self,
        parent_name: Option<&str>,
    ) -> Option<Vec<(String, MethodKind)>> {
        let name = parent_name?;
        let sym = self.scopes.lookup(name)?;
        if let Ty::Interface(parent) = &sym.ty {
            Some(parent.methods.clone())
        } else {
            None
        }
    }

    /// Resolve method signatures in an interface body to `MethodKind` values.
    ///
    /// `Self` in parameters is treated as the interface type itself.
    fn resolve_interface_method_sigs(
        &mut self,
        iface_name: &str,
        iface_ty: &Ty,
        methods: &[RecordMethod],
    ) -> Vec<(String, MethodKind)> {
        let mut result = Vec::new();
        for method in methods {
            match method {
                RecordMethod::Function(f) => {
                    let return_ty =
                        self.resolve_method_param_type(&f.return_type, iface_name, iface_ty);
                    let params: Vec<ParamTy> = f
                        .params
                        .iter()
                        .map(|p| ParamTy {
                            mutable: p.mutable,
                            name: p.name.clone(),
                            ty: self
                                .resolve_method_param_type(&p.type_expr, iface_name, iface_ty),
                        })
                        .collect();
                    result.push((
                        f.name.clone(),
                        MethodKind::Function(FunctionTy {
                            type_params: Vec::new(),
                            params,
                            return_type: Box::new(return_ty),
                        }),
                    ));
                }
                RecordMethod::Procedure(p) => {
                    let params: Vec<ParamTy> = p
                        .params
                        .iter()
                        .map(|param| ParamTy {
                            mutable: param.mutable,
                            name: param.name.clone(),
                            ty: self.resolve_method_param_type(
                                &param.type_expr,
                                iface_name,
                                iface_ty,
                            ),
                        })
                        .collect();
                    result.push((
                        p.name.clone(),
                        MethodKind::Procedure(ProcedureTy {
                            type_params: Vec::new(),
                            variadic: false,
                            params,
                        }),
                    ));
                }
            }
        }
        result
    }
}
