//! Bounded portable source identities copied before the VM owns its image.

use fpas_bytecode::VerifiedExecutable;
use std::{collections::BTreeSet, path::PathBuf};

/// Source identities contain no host directory or filesystem lookup.
pub(crate) struct Sources {
    total: usize,
    names: Vec<Option<String>>,
    /// Metadata omissions that the report must disclose.
    pub(super) reasons: BTreeSet<&'static str>,
}

impl Sources {
    /// Copy within a fixed metadata allowance, never copying a complete unbounded path first.
    pub(crate) fn new(image: &VerifiedExecutable, paths: Option<&[PathBuf]>) -> Self {
        let image = image.executable();
        let mut result = Self {
            total: image.source_map.sources.len(),
            names: Vec::new(),
            reasons: BTreeSet::new(),
        };
        let mut remaining = 1024 * 1024;
        for (index, source) in image.source_map.sources.iter().enumerate() {
            let stored = image.strings.get(*source);
            let actual = paths
                .and_then(|paths| paths.get(index))
                .and_then(|path| path.to_str());
            let name = actual.or(stored).filter(|name| !name.is_empty());
            let name = name.map(|name| {
                let absolute = name.starts_with(['/', '\\'])
                    || name.as_bytes().get(1) == Some(&b':')
                    || name.split(['/', '\\']).any(|part| part == "..");
                let portable = if absolute {
                    name.rsplit(['/', '\\']).next().unwrap_or("")
                } else {
                    name
                };
                if portable.chars().nth(4096).is_some() {
                    result.reasons.insert("string_limit");
                }
                portable
                    .chars()
                    .take(4096)
                    .map(|ch| if ch == '\\' { '/' } else { ch })
                    .collect::<String>()
            });
            let cost = std::mem::size_of::<Option<String>>() + name.as_ref().map_or(0, String::len);
            if cost > remaining {
                result.reasons.insert("retained_byte_limit");
                break;
            }
            remaining -= cost;
            result.names.push(name);
        }
        result
    }

    /// Return only metadata actually retained in the image or its known source mapping.
    pub(super) fn get(&self, id: u32) -> Option<&str> {
        self.names.get(id as usize).and_then(|name| name.as_deref())
    }

    /// Distinguish an image source identity from a runtime fallback coordinate.
    pub(super) fn contains(&self, id: u32) -> bool {
        (id as usize) < self.total
    }
}
