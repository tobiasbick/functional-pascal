//! Standard library units (`Std.*`) referenced only via `uses`.

use fpas_parser::QualifiedId;
use fpas_std::{
    STD_UNITS_KNOWN, canonical_std_unit_from_segments, canonical_std_unit_from_tail,
    is_std_root_segment,
};
use std::collections::HashSet;

/// `true` if `name` is at least `Std.<something>` (case-insensitive on `Std`).
pub fn looks_like_std_qualified_name(name: &str) -> bool {
    let Some((head, tail)) = name.split_once('.') else {
        return false;
    };
    is_std_root_segment(head) && !tail.is_empty()
}

/// Human-readable list for error hints.
pub fn std_units_list_for_hint() -> String {
    fpas_std::std_units_list_for_hint()
}

/// `true` if `ident` is the reserved standard-library root segment (`Std`, any ASCII case).
///
/// User-defined units must not use this as their first name segment; only the implementation may define `Std.*`.
pub fn is_reserved_std_root_segment(ident: &str) -> bool {
    is_std_root_segment(ident)
}

/// Map a `uses` clause entry to a canonical std unit name, or an error message.
pub fn canonical_unit_from_uses_clause(q: &QualifiedId) -> Result<String, String> {
    let display = q.parts.join(".");

    if q.parts.is_empty() {
        return Err("Empty `uses` entry.".to_string());
    }

    let root_is_std = is_reserved_std_root_segment(&q.parts[0]);

    if q.parts.len() == 1 {
        if root_is_std {
            return Err(
                "The name `Std` is reserved for the standard library namespace. Only the language implementation may define units under `Std.*`; user code must not declare or import a bare `Std` unit. Use a concrete library unit such as `Std.Console` in `uses`."
                    .to_string(),
            );
        }
        return Err(format!(
            "Invalid `uses` entry `{display}`: expected a two-part name such as `Std.Console`."
        ));
    }

    if root_is_std && q.parts.len() != 2 {
        return Err(format!(
            "Invalid `uses` entry `{display}`: the reserved namespace `Std` must be followed by exactly one segment (for example `Std.Console`). User code cannot add extra segments after `Std` in a `uses` clause."
        ));
    }

    if q.parts.len() != 2 {
        return Err(format!(
            "Invalid `uses` entry `{display}`: expected a two-part standard library name such as `Std.Console`."
        ));
    }

    if !root_is_std {
        return Err(format!(
            "Unknown unit `{display}`. Only standard library units exist for now: {}.",
            std_units_list_for_hint()
        ));
    }
    let tail = q.parts[1].as_str();
    let Some(canon) = canonical_std_unit_from_segments(&q.parts[0], tail) else {
        return Err(format!(
            "Unknown standard library unit `Std.{tail}`. Available: {}.",
            std_units_list_for_hint()
        ));
    };
    Ok(canon.to_string())
}

/// If `name` looks like `Std.<Unit>.<member>`, returns canonical unit and full member path (segments after unit).
pub fn parse_std_qualified_call(name: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() < 3 || parts.iter().any(|s| s.is_empty()) {
        return None;
    }
    if !is_std_root_segment(parts[0]) {
        return None;
    }
    let unit = canonical_std_unit_from_tail(parts[1])?;
    let member = parts[2..].join(".");
    Some((unit.to_string(), member))
}

/// LLM-friendly hint when a call or identifier is missing from scope.
pub fn hint_for_unknown_std_name(name: &str, loaded: &HashSet<String>) -> String {
    let parsed = parse_std_qualified_call(name);
    let parts: Vec<&str> = name.split('.').collect();

    // Fully-qualified Std.* call
    if parts.len() >= 3 && is_std_root_segment(parts[0]) && parsed.is_none() {
        return format!(
            "Unknown standard library unit in `{name}`. Valid `Std.*` units: {}.",
            std_units_list_for_hint()
        );
    }

    if let Some((unit, _member)) = parsed {
        if !loaded.contains(&unit) {
            return format!(
                "Add `uses {unit};` immediately after the program name (before constants, variables, or `begin`). Example: `program Main; uses {unit}; begin ... end.`"
            );
        }
        return format!(
            "The standard unit `{unit}` is listed in `uses`, but `{name}` is not implemented in the runtime. Check docs/pascal/11-stdlib.md for supported members."
        );
    }

    // Short (unqualified) name — check if it belongs to any known unit
    for unit in STD_UNITS_KNOWN {
        let candidate = format!("{unit}.{name}");
        if let Some((u, _)) = parse_std_qualified_call(&candidate)
            && !loaded.contains(&u)
        {
            return format!("`{name}` may be `{candidate}`. Add `uses {u};` to import the unit.");
        }
    }

    "Declare the function or procedure before use, or check the spelling.".to_string()
}
