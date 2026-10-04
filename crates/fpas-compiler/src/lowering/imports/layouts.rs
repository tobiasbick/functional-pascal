//! Layout imports needed by transitive nominal types.

use super::{
    BTreeSet, ImportPlan, ImportShape, InterfaceType, ObjectImport, SymbolKind, UnitInterface,
};

/// Retain only layouts provided by an interface in the supporting closure.
pub(super) fn collect_layouts(interfaces: &[UnitInterface], plan: &mut ImportPlan) {
    let providers = interfaces
        .iter()
        .flat_map(|interface| &interface.symbols)
        .filter(|symbol| symbol.kind == SymbolKind::Type)
        .map(|symbol| symbol.qualified_name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for symbol in interfaces
        .iter()
        .flat_map(|interface| &interface.symbols)
        .filter(|symbol| symbol.kind == SymbolKind::Type)
    {
        let (name, shape) = match &symbol.ty {
            InterfaceType::Record(record)
                if providers.contains(&record.name.to_ascii_lowercase()) =>
            {
                (
                    &record.name,
                    Some(ImportShape::Record {
                        fields: record
                            .fields
                            .iter()
                            .map(|field| field.name.to_ascii_lowercase())
                            .collect(),
                    }),
                )
            }
            InterfaceType::Enum(enumeration)
                if enumeration
                    .variants
                    .iter()
                    .any(|variant| !variant.fields.is_empty()) =>
            {
                if providers.contains(&enumeration.name.to_ascii_lowercase()) {
                    (
                        &enumeration.name,
                        Some(ImportShape::Enum {
                            variants: enumeration
                                .variants
                                .iter()
                                .map(|variant| {
                                    (
                                        variant.name.to_ascii_lowercase(),
                                        variant
                                            .fields
                                            .iter()
                                            .map(|field| field.name.to_ascii_lowercase())
                                            .collect(),
                                    )
                                })
                                .collect(),
                        }),
                    )
                } else {
                    (&enumeration.name, None)
                }
            }
            InterfaceType::Enum(enumeration) => (&enumeration.name, None),
            _ => (&symbol.qualified_name, None),
        };
        if let Some(shape) = shape {
            plan.layouts.push(ObjectImport {
                name: name.to_ascii_lowercase(),
                shape,
            });
        }
    }
}
