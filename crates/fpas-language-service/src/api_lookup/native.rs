//! Lookup in the fixed catalog of built-in type operations.

use fpas_sema::{NativeOperation, NativeReceiver, native_operations};

use super::{ApiEntry, ApiEntryKind};

/// Finds `Owner.Name` or a bare `Name` among built-in operations, case-insensitively.
pub(super) fn matches(query: &str) -> Vec<ApiEntry> {
    let (owner, name) = match query.rsplit_once('.') {
        Some((owner, name)) => (Some(owner), name),
        None => (None, query),
    };
    native_operations()
        .filter(|entry| entry.name.eq_ignore_ascii_case(name))
        .filter(|entry| owner.is_none_or(|owner| owner_matches(entry.receiver, owner)))
        .map(entry)
        .collect()
}

/// Every operation as `owner.Name`, for suggestions.
pub(super) fn names() -> impl Iterator<Item = String> {
    native_operations().map(qualified_name)
}

fn owner_matches(receiver: NativeReceiver, owner: &str) -> bool {
    // `Join` exists only on `array of string`, which is written and looked up as `array`.
    let owner = owner.trim();
    receiver.label().eq_ignore_ascii_case(owner)
        || (receiver == NativeReceiver::StringArray && owner.eq_ignore_ascii_case("array"))
}

pub(crate) fn qualified_name(entry: &NativeOperation) -> String {
    let owner = match entry.receiver {
        NativeReceiver::StringArray => "array",
        receiver => receiver.label(),
    };
    format!("{owner}.{}", entry.name)
}

fn entry(entry: &'static NativeOperation) -> ApiEntry {
    let mut documentation = entry.documentation.to_owned();
    if entry.receiver == NativeReceiver::StringArray {
        documentation.push_str("\n\nAvailable on `array of string` only.");
    }
    ApiEntry {
        name: qualified_name(entry),
        kind: if entry.receiver.is_factory() {
            ApiEntryKind::Factory
        } else {
            ApiEntryKind::Operation
        },
        signature: crate::intellisense::native::signature(entry, None).label,
        documentation: Some(documentation),
        handbook: Some(entry.receiver.handbook().to_owned()),
        location: None,
        members: Vec::new(),
    }
}
