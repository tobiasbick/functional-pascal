//! Resolves `uses` units and builds short-name → qualified `Std.*` aliases.
//!
//! **Documentation:** `docs/pascal/09-units.md` (from the repository root).

use std::collections::HashMap;

use fpas_parser::Program;
use fpas_std::key_event::KEY_KIND_VARIANTS;
use fpas_std::{
    EVENT_KIND_VARIANTS, MOUSE_ACTION_VARIANTS, MOUSE_BUTTON_VARIANTS, STD_UNIT_CONSOLE,
    STD_UNIT_TUI, TUI_EVENT_KIND_VARIANTS, canonical_std_unit_from_segments, std_unit_symbols,
};

use super::{Compiler, canonical_name};

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
        let units: Vec<String> = program
            .uses
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
            } else if unit == STD_UNIT_TUI {
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_TUI,
                    "KeyKind",
                    KEY_KIND_VARIANTS,
                );
                record_enum_member_short_names(
                    &mut seen,
                    STD_UNIT_TUI,
                    "EventKind",
                    TUI_EVENT_KIND_VARIANTS,
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
