//! Resolves `uses` units and builds short-name → qualified `Std.*` aliases.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md` (from the repository root).

use std::collections::HashMap;

use fpas_parser::{Decl, Program, QualifiedId, RecordMethod, TypeBody, Unit};
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::{
    CONSOLE_COLOR_KIND_VARIANTS, EVENT_KIND_VARIANTS, GRAPH_EVENT_KIND_VARIANTS,
    MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS, STD_UNIT_CONSOLE, STD_UNIT_GRAPH, STD_UNIT_JSON,
    STD_UNIT_TOML, canonical_std_unit_from_segments, std_unit_symbols,
};

use super::{Compiler, canonical_name};
use fpas_unit::interface::{InterfaceType, SymbolKind, UnitInterface};

fn record_enum_member_short_names(
    seen: &mut HashMap<String, Vec<String>>,
    unit: &str,
    enum_name: &str,
    variants: &[&str],
) {
    for &variant in variants {
        let short = format!("{enum_name}.{variant}");
        let qualified = format!("{unit}.{enum_name}.{variant}");
        seen.entry(short).or_default().push(qualified);
    }
}

impl Compiler {
    /// Build short-name → qualified-name aliases from the `uses` clause.
    pub(super) fn build_short_aliases(&mut self, program: &Program) {
        self.build_std_short_aliases(&program.uses);
    }

    pub(super) fn build_unit_short_aliases(&mut self, unit: &Unit, interfaces: &[UnitInterface]) {
        self.build_std_short_aliases(&unit.uses);
        self.add_interface_short_aliases(interfaces);
        let owner = unit.name.parts.join(".");
        for declaration in &unit.declarations {
            let name = declaration_name(declaration);
            self.short_aliases.insert(
                canonical_name(name),
                format!("{owner}.{name}").to_ascii_lowercase(),
            );
            if let Decl::TypeDef(definition) = declaration
                && let TypeBody::Record(record) = &definition.body
            {
                for method in &record.methods {
                    let method_name = match method {
                        RecordMethod::Function(value) | RecordMethod::StaticFunction(value) => {
                            &value.name
                        }
                        RecordMethod::Procedure(value) | RecordMethod::StaticProcedure(value) => {
                            &value.name
                        }
                    };
                    self.short_aliases.insert(
                        canonical_name(&format!("{}.{}", definition.name, method_name)),
                        format!("{owner}.{}.{}", definition.name, method_name).to_ascii_lowercase(),
                    );
                }
            }
        }
    }

    pub(super) fn build_program_interface_aliases(&mut self, interfaces: &[UnitInterface]) {
        self.add_interface_short_aliases(interfaces);
    }

    fn add_interface_short_aliases(&mut self, interfaces: &[UnitInterface]) {
        let mut seen = HashMap::<String, Vec<String>>::new();
        for interface in interfaces {
            for symbol in &interface.symbols {
                self.short_aliases.insert(
                    canonical_name(&symbol.qualified_name),
                    symbol.qualified_name.clone(),
                );
                seen.entry(canonical_name(&symbol.name))
                    .or_default()
                    .push(symbol.qualified_name.clone());
                if matches!(
                    symbol.kind,
                    SymbolKind::Constant(_) | SymbolKind::Variable | SymbolKind::MutableVariable
                ) {
                    self.module_globals
                        .insert(canonical_name(&symbol.qualified_name));
                }
                if matches!(symbol.kind, SymbolKind::Function | SymbolKind::Procedure) {
                    self.external_callables
                        .insert(canonical_name(&symbol.qualified_name));
                }
                if let InterfaceType::Enum(enum_ty) = &symbol.ty {
                    for variant in &enum_ty.variants {
                        let qualified = format!("{}.{}", enum_ty.name, variant.name);
                        seen.entry(canonical_name(&variant.name))
                            .or_default()
                            .push(qualified.clone());
                        seen.entry(canonical_name(&format!("{}.{}", symbol.name, variant.name)))
                            .or_default()
                            .push(qualified);
                    }
                } else if let InterfaceType::Record(record) = &symbol.ty {
                    for method in record.methods.iter().chain(&record.static_routines) {
                        seen.entry(canonical_name(&format!("{}.{}", symbol.name, method.name)))
                            .or_default()
                            .push(format!("{}.{}", record.name, method.name));
                    }
                }
            }
        }
        for (short, mut candidates) in seen {
            candidates.sort_by_key(|candidate| canonical_name(candidate));
            candidates.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
            if let [qualified] = candidates.as_slice() {
                self.short_aliases.insert(short, qualified.clone());
            }
        }
    }

    fn build_std_short_aliases(&mut self, uses: &[QualifiedId]) {
        let units: Vec<String> = uses
            .iter()
            .filter_map(|u| {
                if u.parts.len() != 2 {
                    return None;
                }
                canonical_std_unit_from_segments(&u.parts[0], &u.parts[1]).map(str::to_string)
            })
            .collect();

        // Collect all short → qualified mappings; track ambiguous ones.
        let mut seen: HashMap<String, Vec<String>> = HashMap::new();
        for unit in &units {
            for &qname in std_unit_symbols(unit) {
                self.short_aliases
                    .insert(canonical_name(qname), qname.to_string());
                let prefix = format!("{unit}.");
                if let Some(short) = qname.strip_prefix(&prefix) {
                    seen.entry(short.to_string())
                        .or_default()
                        .push(qname.to_string());
                }
            }
            // Also register enum member short aliases (e.g. KeyKind.Space → Std.Console.KeyKind.Space).
            if unit == STD_UNIT_CONSOLE {
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_CONSOLE,
                    "ColorKind",
                    CONSOLE_COLOR_KIND_VARIANTS,
                );
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_CONSOLE,
                    "KeyKind",
                    KEY_KIND_VARIANTS,
                );
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_CONSOLE,
                    "EventKind",
                    EVENT_KIND_VARIANTS,
                );
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_CONSOLE,
                    "MouseAction",
                    MOUSE_ACTION_VARIANTS,
                );
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_CONSOLE,
                    "MouseButton",
                    MOUSE_BUTTON_VARIANTS,
                );
            } else if unit == STD_UNIT_GRAPH {
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_GRAPH,
                    "EventKind",
                    GRAPH_EVENT_KIND_VARIANTS,
                );
            } else if unit == STD_UNIT_JSON {
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_JSON,
                    "JsonValue",
                    &["Null", "Bool", "Number", "String", "Array", "Object"],
                );
            } else if unit == STD_UNIT_TOML {
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_TOML,
                    "TomlValue",
                    &[
                        "String", "Integer", "Float", "Boolean", "Datetime", "Array", "Table",
                    ],
                );
            }
        }

        // Only register unambiguous aliases.
        for (short, qualified) in seen {
            if let [qualified_name] = qualified.as_slice() {
                self.short_aliases
                    .insert(canonical_name(&short), qualified_name.clone());
            }
        }
    }

    /// Resolve a possibly-short name to its fully-qualified equivalent.
    pub(super) fn qualify_name<'a>(&'a self, name: &'a str) -> &'a str {
        self.short_aliases
            .get(&canonical_name(name))
            .map(|s| s.as_str())
            .unwrap_or(name)
    }
}

fn declaration_name(declaration: &Decl) -> &str {
    match declaration {
        Decl::Const(value) => &value.name,
        Decl::Var(value) | Decl::MutableVar(value) => &value.name,
        Decl::TypeDef(value) => &value.name,
        Decl::Function(value) => &value.name,
        Decl::Procedure(value) => &value.name,
    }
}
