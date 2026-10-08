# AP13.1: Reserve elsif, when, and null

Package: [AP13: Explicit block boundaries](README.md)

Status: complete.

## Result

`elsif`, `when`, and `null` are reserved case-insensitively. Invalid identifier
uses receive rename hints and parser recovery. Highlighting and snippets use
the current keyword set.

The JSON null variant is `JsonValue.NullValue` in signatures, runtime, editor
API and documentation. JSON and TOML data text retain their ordinary meaning.

## Regression coverage

Lexer, parser, stdlib and editor regressions cover keyword cases, invalid names,
recovery, JSON null round trips and unaffected data text.
See [keywords](../../../pascal/getting-started/keywords.md).
