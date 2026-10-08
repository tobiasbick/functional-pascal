# AP02.5: Diagnostics reference

Package: [AP02: Structured diagnostics](README.md)

Status: complete.

## Result

The [diagnostics reference](../../../pascal/tools/diagnostics.md) documents
the shared record, coordinate rules, JSON envelope, and every allocated code
with cause, incorrect use, and correction. Invariant failures use artifact or
host-state examples when valid source cannot directly trigger them.

## Regression coverage

Catalog tests require each allocated code to appear exactly once in the
reference. Producer and CLI tests check representative wrong/corrected cases.
Owning APs extend this catalog and reference when they add diagnostics.
