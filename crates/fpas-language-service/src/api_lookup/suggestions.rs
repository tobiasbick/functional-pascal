//! Closest-name suggestions for a lookup without matches.

const SUGGESTION_LIMIT: usize = 5;

/// Returns up to five known names closest to the query, compared case-insensitively.
///
/// A query with `n` dotted segments is compared with the last `n` segments of each name, so
/// `Fs.ReadTxt` suggests `Std.Fs.ReadText`.
pub(super) fn closest(query: &str, names: impl Iterator<Item = String>) -> Vec<String> {
    let query = query.to_ascii_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let segments = query.split('.').count();
    let limit = (query.chars().count() / 3).max(2);
    let mut scored = names
        .filter_map(|name| {
            let lower = name.to_ascii_lowercase();
            let parts = lower.split('.').collect::<Vec<_>>();
            let tail = parts[parts.len().saturating_sub(segments)..].join(".");
            let distance = edit_distance(&query, &tail);
            (distance <= limit).then_some((distance, name))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        left.0.cmp(&right.0).then_with(|| {
            left.1
                .to_ascii_lowercase()
                .cmp(&right.1.to_ascii_lowercase())
        })
    });
    let mut names = Vec::new();
    for (_, name) in scored {
        if !names
            .iter()
            .any(|known: &String| known.eq_ignore_ascii_case(&name))
        {
            names.push(name);
        }
        if names.len() == SUGGESTION_LIMIT {
            break;
        }
    }
    names
}

// Levenshtein distance over Unicode scalars.
fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (i, left_char) in left.chars().enumerate() {
        let mut current = vec![i + 1; right.len() + 1];
        for (j, right_char) in right.iter().enumerate() {
            let substitution = previous[j] + usize::from(left_char != *right_char);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance_counts_insertions_deletions_and_substitutions() {
        assert_eq!(edit_distance("readtext", "readtext"), 0);
        assert_eq!(edit_distance("readtxt", "readtext"), 1);
        assert_eq!(edit_distance("trimleft", "trimlfet"), 2);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    #[test]
    fn suggestions_compare_the_same_number_of_trailing_segments() {
        let names = ["Std.Fs.ReadText", "Std.Fs.WriteText", "Std.Path.Join"]
            .into_iter()
            .map(String::from);
        assert_eq!(closest("Fs.ReadTxt", names), ["Std.Fs.ReadText"]);
    }

    #[test]
    fn distant_names_are_not_suggested() {
        let names = ["Std.Fs.ReadText"].into_iter().map(String::from);
        assert!(closest("Console", names).is_empty());
    }
}
