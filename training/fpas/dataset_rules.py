"""Shared content rules for the Functional Pascal training dataset."""

from __future__ import annotations

import re


SOURCE_ROOTS = ("docs/pascal", "examples", "tests", "apps", "lib/Std")

FORBIDDEN_CONTENT = (
    (re.compile(r"\bStd\.(?:Array|Dict|Option|Result)\b"), "legacy Std.* unit name"),
    (
        re.compile(
            r"\bStd\.Net\.(?:Read|Write)(?:WithCancellation)?\b"
            r"|\bStd\.Console\.(?:Read|Write)\b"
        ),
        "legacy reserved-word API name",
    ),
    (re.compile(r"\bTuiCmdOutput\.Read\b"), "legacy TUI command accessor"),
    (
        re.compile(r"\b(?:JsonValue|TomlValue)\.Array\b"),
        "legacy reserved-word enum variant",
    ),
    (
        re.compile(
            r"\b(?:ReadEvent|ReadEventTimeout|PollEvent)\([^\n)]*\)"
            r"\s*:\s*(?:Option\s+of\s+)?Event\b",
            re.IGNORECASE,
        ),
        "legacy reserved Event type",
    ),
    (
        re.compile(
            r"^\s*(?:var|mutable\s+var)\s+[A-Za-z_][A-Za-z0-9_]*"
            r"\s*:\s*(?:Option\s+of\s+)?Event\s*:=\s*"
            r"(?:ReadEvent|ReadEventTimeout|PollEvent)\b",
            re.IGNORECASE | re.MULTILINE,
        ),
        "legacy reserved Event type",
    ),
    (
        re.compile(r"^\s*type\s+Event\s*=\s*record\b", re.IGNORECASE | re.MULTILINE),
        "legacy reserved Event type",
    ),
    (
        re.compile(r"^\s*match\s+.+\s+with\s*$", re.IGNORECASE | re.MULTILINE),
        "foreign match-with syntax",
    ),
    (
        re.compile(
            r"`(?:unit_decl|program_decl|const_decl|type_decl|generic_params|constraint|"
            r"result_type|option_type|go_expr)`"
        ),
        "nonexistent grammar production reference",
    ),
    (
        re.compile(r"\bintegers,\s*chars,\s*strings,\s*booleans\b", re.IGNORECASE),
        "removed char type wording",
    ),
    (
        re.compile(r"`string`,\s*`string`,\s*`#` codes", re.IGNORECASE),
        "duplicate string primitive",
    ),
)


def forbidden_reason(content: str) -> str | None:
    """Return the first reason that makes content unsuitable as positive FPAS data."""
    for pattern, reason in FORBIDDEN_CONTENT:
        if pattern.search(content):
            return reason
    return None
