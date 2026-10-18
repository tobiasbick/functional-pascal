//! Human-readable `fpas doc` output, one block per match.

use std::io::{self, Write};
use std::path::Path;

use fpas_language_service::{ApiEntry, ApiLookup};

use super::{display_path, kind_name};

/// Writes every match; writes nothing when there is none.
pub(super) fn write(stdout: &mut dyn Write, lookup: &ApiLookup, cwd: &Path) -> io::Result<()> {
    for (index, entry) in lookup.matches.iter().enumerate() {
        if index > 0 {
            writeln!(stdout)?;
        }
        write_entry(stdout, entry, cwd)?;
    }
    Ok(())
}

fn write_entry(stdout: &mut dyn Write, entry: &ApiEntry, cwd: &Path) -> io::Result<()> {
    writeln!(stdout, "{} ({})", entry.name, kind_name(entry.kind))?;
    writeln!(stdout, "  {}", entry.signature)?;
    if let Some(documentation) = &entry.documentation {
        writeln!(stdout)?;
        for line in documentation.lines() {
            if line.is_empty() {
                writeln!(stdout)?;
            } else {
                writeln!(stdout, "  {line}")?;
            }
        }
    }
    if !entry.members.is_empty() {
        writeln!(stdout)?;
        writeln!(stdout, "  Declarations:")?;
        for member in &entry.members {
            writeln!(stdout, "    {}", member.signature)?;
            if let Some(summary) = &member.summary {
                writeln!(stdout, "      {summary}")?;
            }
        }
    }
    // A unit comment usually names its handbook page already.
    let handbook = entry.handbook.as_ref().filter(|handbook| {
        !entry
            .documentation
            .as_deref()
            .is_some_and(|documentation| documentation.contains(handbook.as_str()))
    });
    if entry.location.is_some() || handbook.is_some() {
        writeln!(stdout)?;
    }
    if let Some(location) = &entry.location {
        writeln!(
            stdout,
            "  Declared at {}:{}:{}",
            display_path(&location.path, cwd).display(),
            location.line,
            location.column
        )?;
    }
    if let Some(handbook) = handbook {
        writeln!(stdout, "  Handbook: {handbook}")?;
    }
    Ok(())
}
