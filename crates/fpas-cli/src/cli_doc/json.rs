//! Machine-readable `fpas doc --report json` output: one JSON document on stdout.

use std::io::{self, Write};
use std::path::Path;

use fpas_language_service::{ApiEntry, ApiLookup};
use serde::Serialize;

use super::{display_path, kind_name};

#[derive(Serialize)]
struct Report<'a> {
    query: &'a str,
    matches: Vec<Match<'a>>,
    suggestions: &'a [String],
}

#[derive(Serialize)]
struct Match<'a> {
    name: &'a str,
    kind: &'static str,
    signature: &'a str,
    documentation: Option<&'a str>,
    handbook: Option<&'a str>,
    location: Option<Location>,
    members: Vec<Member<'a>>,
}

#[derive(Serialize)]
struct Location {
    path: String,
    line: usize,
    column: usize,
}

#[derive(Serialize)]
struct Member<'a> {
    name: &'a str,
    signature: &'a str,
    summary: Option<&'a str>,
}

/// Writes the complete report followed by a newline.
pub(super) fn write(stdout: &mut dyn Write, lookup: &ApiLookup, cwd: &Path) -> io::Result<()> {
    let report = Report {
        query: &lookup.query,
        matches: lookup
            .matches
            .iter()
            .map(|entry| entry_match(entry, cwd))
            .collect(),
        suggestions: &lookup.suggestions,
    };
    serde_json::to_writer(&mut *stdout, &report).map_err(io::Error::other)?;
    writeln!(stdout)
}

fn entry_match<'a>(entry: &'a ApiEntry, cwd: &Path) -> Match<'a> {
    Match {
        name: &entry.name,
        kind: kind_name(entry.kind),
        signature: &entry.signature,
        documentation: entry.documentation.as_deref(),
        handbook: entry.handbook.as_deref(),
        location: entry.location.as_ref().map(|location| Location {
            path: display_path(&location.path, cwd)
                .to_string_lossy()
                .replace('\\', "/"),
            line: location.line,
            column: location.column,
        }),
        members: entry
            .members
            .iter()
            .map(|member| Member {
                name: &member.name,
                signature: &member.signature,
                summary: member.summary.as_deref(),
            })
            .collect(),
    }
}
