//! Content-derived identities; editor versions and report limits are not semantic inputs.

pub(crate) mod encoding;
#[cfg(test)]
mod tests;

use encoding::CanonicalHasher;
use fpas_project::source_inputs::InputInventory;
use serde::{Deserialize, Serialize};
use std::{io, path::Path};

/// Shared by CLI and LSP built with the same analysis sources and configuration.
pub const ANALYSIS_BUILD_ID: &str = env!("FPAS_ANALYSIS_BUILD_ID");

/// A content digest suitable for portable source references.
pub fn source_digest(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Lossless absolute path spelling for identity records, distinct from display paths.
pub fn identity_path(path: &Path) -> io::Result<String> {
    if !path.is_absolute() {
        return Err(io::Error::other("Identity path must be absolute."));
    }
    let text = path
        .to_str()
        .ok_or_else(|| io::Error::other("unsupported_path_identity"))?;
    #[cfg(windows)]
    let text = if let Some(unc) = text.strip_prefix(r"\\?\UNC\") {
        format!("//{unc}").replace('\\', "/")
    } else {
        text.strip_prefix(r"\\?\")
            .unwrap_or(text)
            .replace('\\', "/")
    };
    #[cfg(not(windows))]
    let text = text.to_owned();
    Ok(text)
}

/// Bounded, serializable metadata for one exact input, without its retained source bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputIdentity {
    /// Lossless physical path, never a relative display path.
    pub path: String,
    /// Lowercase SHA-256 of the exact input bytes.
    pub sha256: String,
    /// Byte count before any decoding or line-ending conversion.
    pub bytes: usize,
}

/// Complete inventory fingerprint. Callers must verify the read set before issuing references.
pub fn inventory_digest(inventory: &InputInventory) -> io::Result<String> {
    Ok(InventoryIdentity::from_inputs(inventory)?.digest())
}

/// Portable metadata for the complete read set; no retained source bytes or editor revisions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryIdentity {
    /// Physical input byte hashes, sorted by identity path.
    pub files: Vec<InputIdentity>,
    /// Sorted lexical/physical identity pairs.
    pub aliases: Vec<(String, String)>,
    /// Sorted admitted physical paths.
    pub selected: Vec<String>,
    /// Sorted discovery patterns, including empty memberships.
    pub globs: Vec<GlobIdentity>,
}

/// One canonical discovery selection.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlobIdentity {
    /// Absolute lexical discovery root.
    pub root: String,
    /// Original include pattern.
    pub pattern: String,
    /// Ordered exclusion patterns.
    pub exclude: Vec<String>,
    /// Sorted selected lexical paths.
    pub members: Vec<String>,
}

impl InventoryIdentity {
    /// Projects a complete owned read set into lossless portable metadata.
    pub fn from_inputs(inventory: &InputInventory) -> io::Result<Self> {
        let mut files = inventory
            .files()
            .iter()
            .map(|file| {
                Ok(InputIdentity {
                    path: identity_path(&file.path)?,
                    sha256: source_digest(&file.bytes),
                    bytes: file.bytes.len(),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut aliases = inventory
            .aliases()
            .iter()
            .map(|alias| {
                Ok((
                    identity_path(&alias.lexical)?,
                    identity_path(&alias.physical)?,
                ))
            })
            .collect::<io::Result<Vec<_>>>()?;
        aliases.sort();
        aliases.dedup();
        let mut selected = inventory
            .selected()
            .iter()
            .map(|p| identity_path(p))
            .collect::<io::Result<Vec<_>>>()?;
        selected.sort();
        selected.dedup();
        let mut globs = inventory
            .globs()
            .iter()
            .map(|glob| {
                let mut members = glob
                    .members
                    .iter()
                    .map(|p| identity_path(&std::path::absolute(p)?))
                    .collect::<io::Result<Vec<_>>>()?;
                members.sort();
                Ok(GlobIdentity {
                    root: identity_path(&std::path::absolute(&glob.root)?)?,
                    pattern: glob.pattern.clone(),
                    exclude: glob.exclude.clone(),
                    members,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        globs.sort();
        // A workspace member can also be reached as a dependency. Platform-equivalent
        // spellings may be distinct loader keys but describe one identical selection.
        globs.dedup();
        Ok(Self {
            files,
            aliases,
            selected,
            globs,
        })
    }

    /// Hashes canonical metadata; callers accepting external plans also validate structure.
    pub fn digest(&self) -> String {
        let mut hash = CanonicalHasher::new("fpas-source-inventory-v1");
        hash.count(self.files.len());
        for file in &self.files {
            hash.text(&file.path);
            hash.text(&file.sha256);
            hash.count(file.bytes);
        }
        hash.count(self.aliases.len());
        for (lexical, physical) in &self.aliases {
            hash.text(lexical);
            hash.text(physical);
        }
        hash.count(self.selected.len());
        for path in &self.selected {
            hash.text(path);
        }
        hash.count(self.globs.len());
        for glob in &self.globs {
            hash.text(&glob.root);
            hash.text(&glob.pattern);
            hash.count(glob.exclude.len());
            for pattern in &glob.exclude {
                hash.text(pattern);
            }
            hash.count(glob.members.len());
            for member in &glob.members {
                hash.text(member);
            }
        }
        hash.finish()
    }
}
