//! Ordered leading comments and declaration-documentation spacing.

use super::{LeadingComment, PendingLeadingComment};
use std::collections::{BTreeMap, BTreeSet};

/// Orders attached comments and retains declaration-documentation spacing.
pub(super) fn prepare_leading(
    source: &str,
    grouped: BTreeMap<usize, Vec<PendingLeadingComment>>,
    declaration_anchors: &BTreeSet<usize>,
) -> (BTreeMap<usize, Vec<LeadingComment>>, BTreeMap<usize, bool>) {
    let mut leading = BTreeMap::new();
    let mut blank_after = BTreeMap::new();
    for (anchor, mut entries) in grouped {
        entries.sort_by_key(|entry| entry.start);
        let mut previous_end = None;
        let prepared = entries
            .iter()
            .map(|entry| {
                let blank_before = previous_end.is_some_and(|end| {
                    logical_line_breaks(source.get(end..entry.start).unwrap_or_default()) > 1
                });
                previous_end = Some(entry.end);
                LeadingComment {
                    text: entry.text.clone(),
                    blank_before,
                }
            })
            .collect();
        if declaration_anchors.contains(&anchor) {
            let last_end = entries.last().map_or(anchor, |entry| entry.end);
            blank_after.insert(
                anchor,
                logical_line_breaks(source.get(last_end..anchor).unwrap_or_default()) != 1,
            );
        }
        leading.insert(anchor, prepared);
    }
    (leading, blank_after)
}

fn logical_line_breaks(text: &str) -> usize {
    let mut count = 0;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                count += 1;
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            '\n' => count += 1,
            _ => {}
        }
    }
    count
}
