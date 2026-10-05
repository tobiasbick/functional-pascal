//! Unknown-unit failures with a short, relevant list of known units.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use fpas_diagnostics::codes::PROJECT_UNKNOWN_UNIT;

use super::model::UnitGraph;
use crate::ProjectError;
use crate::source::display_unit_key;

// Keeps hints readable when the standard library contributes many units.
const MAX_LISTED_UNITS: usize = 10;

/// Reports a missing unit and lists the known units closest to its namespace.
pub(crate) fn unknown_unit_error(key: &str, graph: &UnitGraph, owner: &str) -> ProjectError {
    let display = display_unit_key(key);
    let known = graph
        .iter()
        .map(|(_, node)| node.display_name().to_string())
        .collect::<Vec<_>>();
    let error = ProjectError::new(
        PROJECT_UNKNOWN_UNIT,
        format!("Unknown unit `{display}` in {owner}."),
    );
    match candidate_hint(&display, known) {
        Some(hint) => error.with_help(hint),
        None => error.with_help(
            "No source units are available in the project. Add a source file that declares the unit, or remove it from `uses`.",
        ),
    }
}

/// Picks units in the same namespace, then the same root segment, then non-`Std` units.
fn candidate_hint(display: &str, mut known: Vec<String>) -> Option<String> {
    known.sort_by_key(|name| name.to_ascii_lowercase());
    let namespace = display.rsplit_once('.').map(|(namespace, _)| namespace);
    let root = display.split('.').next().filter(|_| namespace.is_some());

    for prefix in [namespace, root].into_iter().flatten() {
        let matching = units_in(&known, prefix);
        if !matching.is_empty() {
            return Some(format!("Known units in `{prefix}`: {}.", listed(&matching)));
        }
    }
    let project_units = known
        .iter()
        .filter(|name| !is_standard_unit(name))
        .map(String::as_str)
        .collect::<Vec<_>>();
    if !project_units.is_empty() {
        return Some(format!("Known project units: {}.", listed(&project_units)));
    }
    let all = known.iter().map(String::as_str).collect::<Vec<_>>();
    (!all.is_empty()).then(|| format!("Known units: {}.", listed(&all)))
}

fn units_in<'a>(known: &'a [String], prefix: &str) -> Vec<&'a str> {
    known
        .iter()
        .filter(|name| {
            name.get(..prefix.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
                && name
                    .get(prefix.len()..)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
        .map(String::as_str)
        .collect()
}

fn is_standard_unit(name: &str) -> bool {
    name.split('.')
        .next()
        .is_some_and(|root| root.eq_ignore_ascii_case("std"))
}

fn listed(names: &[&str]) -> String {
    let shown = names
        .iter()
        .take(MAX_LISTED_UNITS)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    match names.len().saturating_sub(MAX_LISTED_UNITS) {
        0 => shown,
        hidden => format!("{shown}, and {hidden} more"),
    }
}

#[cfg(test)]
mod tests {
    use super::candidate_hint;

    fn names(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn same_namespace_units_are_preferred() {
        let hint = candidate_hint(
            "Demo.Missing",
            names(&["Std.Http", "Demo.Feature", "Other.Unit"]),
        );
        assert_eq!(
            hint.as_deref(),
            Some("Known units in `Demo`: Demo.Feature.")
        );
    }

    #[test]
    fn root_segment_is_used_when_the_namespace_has_no_units() {
        let hint = candidate_hint("Demo.Ui.Missing", names(&["Demo.Core", "Std.Http"]));
        assert_eq!(hint.as_deref(), Some("Known units in `Demo`: Demo.Core."));
    }

    #[test]
    fn standard_units_are_omitted_from_the_project_fallback() {
        let hint = candidate_hint("Missing", names(&["Std.Http", "App.Main"]));
        assert_eq!(hint.as_deref(), Some("Known project units: App.Main."));
    }

    #[test]
    fn long_lists_are_truncated() {
        let known = (0..14)
            .map(|index| format!("Std.Tui.U{index:02}"))
            .collect();
        let hint = candidate_hint("Std.Tui.Missing", known).expect("hint");
        assert!(hint.starts_with("Known units in `Std.Tui`: Std.Tui.U00, "));
        assert!(hint.ends_with("Std.Tui.U09, and 4 more."));
    }

    #[test]
    fn an_empty_graph_has_no_candidate_list() {
        assert_eq!(candidate_hint("Demo.Missing", Vec::new()), None);
    }
}
