//! Installation of compiled-unit interface symbols into semantic scopes.

use fpas_parser::{Decl, Expr, Program};
use fpas_unit::interface as artifact;

use crate::check;
use crate::scope::{Symbol, SymbolKind as SemaSymbolKind, canonical_symbol_name};

use super::conversion::{InterfaceConversionError, interface_symbol_to_sema, interface_type_to_ty};
use super::export::declaration_name;

impl check::Checker {
    /// Install only qualified type definitions from transitive supporting interfaces.
    pub(crate) fn install_supporting_interface_types(
        &mut self,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        for interface in interfaces {
            for exported in &interface.symbols {
                if exported.kind != artifact::SymbolKind::Type {
                    continue;
                }
                self.scopes.define_in_root(
                    &exported.qualified_name,
                    interface_symbol_to_sema(exported)?,
                );
            }
        }
        Ok(())
    }

    /// Install directly visible interfaces for one program.
    pub(crate) fn install_interfaces(
        &mut self,
        program: &Program,
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        self.install_interfaces_for_declarations(&program.declarations, interfaces)
    }

    /// Install directly visible interface symbols alongside the given declarations.
    pub(crate) fn install_interfaces_for_declarations(
        &mut self,
        declarations: &[Decl],
        interfaces: &[artifact::UnitInterface],
    ) -> Result<(), InterfaceConversionError> {
        use std::collections::{HashMap, HashSet};

        let own_names: HashSet<String> = declarations
            .iter()
            .map(declaration_name)
            .map(canonical_symbol_name)
            .collect();
        let mut short_candidates = HashMap::<String, Vec<(String, Symbol)>>::new();

        for interface in interfaces {
            for exported in &interface.symbols {
                let symbol = interface_symbol_to_sema(exported)?;
                self.scopes
                    .define_in_root(&exported.qualified_name, symbol.clone());
                self.install_imported_record_defaults(exported);
                if !own_names.contains(&canonical_symbol_name(&exported.name)) {
                    short_candidates
                        .entry(canonical_symbol_name(&exported.name))
                        .or_default()
                        .push((exported.qualified_name.clone(), symbol));
                }
                self.install_imported_enum_variants(exported, &mut short_candidates, &own_names)?;
            }
        }

        for (short, mut candidates) in short_candidates {
            candidates.sort_by(|left, right| {
                canonical_symbol_name(&left.0)
                    .cmp(&canonical_symbol_name(&right.0))
                    .then_with(|| left.0.cmp(&right.0))
            });
            candidates.dedup_by(|left, right| left.0.eq_ignore_ascii_case(&right.0));
            if candidates.len() == 1 {
                if let Some((_, symbol)) = candidates.pop() {
                    self.scopes.define_in_root(&short, symbol);
                }
            } else {
                self.ambiguous_imports.insert(
                    short,
                    candidates
                        .into_iter()
                        .map(|(qualified, _)| qualified)
                        .collect(),
                );
            }
        }
        Ok(())
    }

    fn install_imported_record_defaults(&mut self, exported: &artifact::InterfaceSymbol) {
        let artifact::InterfaceType::Record(record) = &exported.ty else {
            return;
        };
        if !record
            .fields
            .iter()
            .any(|field| field.default_value.is_some())
        {
            return;
        }
        let fields = record
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.clone(),
                    field.default_value.as_ref().map(constant_value_to_expr),
                )
            })
            .collect();
        self.record_defaults.insert(record.name.clone(), fields);
    }

    fn install_imported_enum_variants(
        &mut self,
        exported: &artifact::InterfaceSymbol,
        short_candidates: &mut std::collections::HashMap<String, Vec<(String, Symbol)>>,
        own_names: &std::collections::HashSet<String>,
    ) -> Result<(), InterfaceConversionError> {
        let artifact::InterfaceType::Enum(enum_ty) = &exported.ty else {
            return Ok(());
        };
        let enum_symbol_ty = interface_type_to_ty(&exported.ty)?;
        for variant in &enum_ty.variants {
            let kind = if variant.fields.is_empty() {
                SemaSymbolKind::EnumMember
            } else {
                SemaSymbolKind::EnumVariantConstructor
            };
            let symbol = Symbol {
                ty: enum_symbol_ty.clone(),
                mutable: false,
                kind,
                task_bound: false,
            };
            let fully_qualified = format!("{}.{}", enum_ty.name, variant.name);
            self.scopes.define_in_root(&fully_qualified, symbol.clone());
            if !own_names.contains(&canonical_symbol_name(&exported.name)) {
                let type_qualified_short = format!("{}.{}", exported.name, variant.name);
                short_candidates
                    .entry(canonical_symbol_name(&type_qualified_short))
                    .or_default()
                    .push((fully_qualified.clone(), symbol.clone()));
            }
            let short = canonical_symbol_name(&variant.name);
            if !own_names.contains(&short) {
                short_candidates
                    .entry(short)
                    .or_default()
                    .push((fully_qualified, symbol));
            }
        }
        Ok(())
    }
}

fn constant_value_to_expr(value: &artifact::ConstantValue) -> Expr {
    let span = fpas_lexer::Span {
        offset: 0,
        length: 0,
        line: 1,
        column: 1,
        source_id: 0,
    };
    match value {
        artifact::ConstantValue::Integer(value) => Expr::Integer(*value, span),
        artifact::ConstantValue::Real(bits) => Expr::Real(f64::from_bits(*bits), span),
        artifact::ConstantValue::Boolean(value) => Expr::Bool(*value, span),
        artifact::ConstantValue::String(value) => Expr::Str(value.clone(), span),
        artifact::ConstantValue::EnumValue {
            enum_name,
            variant_name,
            ..
        } => Expr::Designator(fpas_parser::Designator {
            parts: enum_name
                .split('.')
                .chain(std::iter::once(variant_name.as_str()))
                .map(|part| fpas_parser::DesignatorPart::Ident(part.to_string(), span))
                .collect(),
            span,
        }),
    }
}
