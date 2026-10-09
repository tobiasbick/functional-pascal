//! Nominal debugger constructors and declaration-bound defaults.
//! See `docs/pascal/language/types/records.md` and `docs/pascal/tools/debugger.md`.

use fpas_ir::{IrType, TypeId};
use fpas_parser::{Decl, Expr};
use std::sync::Arc;

use super::TypeTable;

/// One default body that must be emitted in its declaring source.
pub(in crate::lowering) struct RecordDefault {
    /// Compiler-internal, canonical default routine name.
    pub name: String,
    /// Concrete stored field type.
    pub ty: TypeId,
    /// Semantically checked, declaration-bound expression.
    pub expression: Arc<Expr>,
}

impl TypeTable {
    /// Retains source-visible imported names without replacing a nearer type declaration.
    pub(in crate::lowering) fn register_imported_record_names(
        &mut self,
        qualified: &str,
        short: Option<&str>,
        ty: TypeId,
    ) {
        self.named.insert(qualified.to_ascii_lowercase(), ty);
        self.imported_record_names
            .insert(qualified.to_ascii_lowercase());
        if let Some(short) = short {
            self.named.entry(short.to_ascii_lowercase()).or_insert(ty);
            self.imported_record_names
                .insert(short.to_ascii_lowercase());
        }
    }

    /// Retains visible type aliases and schedules default bodies owned by this source.
    pub(in crate::lowering) fn prepare_record_defaults(
        &mut self,
        metadata: &fpas_sema::AnalysisMetadata,
        scope_unit: &str,
        declarations: &[Decl],
    ) -> Vec<RecordDefault> {
        let declared = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Decl::TypeDef(definition) => Some(definition.name.to_ascii_lowercase()),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        let mut bodies = Vec::new();
        for layout in &mut self.record_layouts {
            let Some(info) = &mut layout.construction else {
                continue;
            };
            info.scope_unit = scope_unit.to_owned();
            info.aliases = self
                .named
                .iter()
                .filter(|(name, _)| {
                    info.owner_unit
                        .as_ref()
                        .is_none_or(|owner| owner.eq_ignore_ascii_case(scope_unit))
                        || self.imported_record_names.contains(*name)
                        || declared.contains(*name)
                })
                .filter_map(|(name, ty)| {
                    self.definitions
                        .get(ty.get() as usize)
                        .filter(|definition| definition.kind == IrType::Record(layout.id))
                        .map(|_| name.clone())
                })
                .collect();
            for visible in &mut info.aliases {
                for (unit, alias) in &metadata.import_aliases {
                    let prefix = format!("{unit}.");
                    if visible
                        .get(..prefix.len())
                        .is_some_and(|name| name.eq_ignore_ascii_case(&prefix))
                    {
                        *visible = format!("{alias}.{}", &visible[prefix.len()..]);
                        break;
                    }
                }
            }
            info.aliases.sort();
            info.aliases.dedup();
            let Some(defaults) = metadata.record_defaults.get(&layout.name) else {
                continue;
            };
            for (index, field) in layout.fields.iter().enumerate() {
                let Some(expression) = defaults
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&field.name))
                    .and_then(|(_, default)| default.clone())
                else {
                    continue;
                };
                let name = format!("{}.@default.{index}", layout.name.to_ascii_lowercase());
                info.defaults[index] = Some(name.clone());
                if info
                    .owner_unit
                    .as_ref()
                    .is_none_or(|owner| owner.eq_ignore_ascii_case(scope_unit))
                {
                    bodies.push(RecordDefault {
                        name,
                        ty: field.ty,
                        expression,
                    });
                }
            }
        }
        bodies
    }
}
