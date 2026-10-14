# AP44.1: Enforce unambiguous enum values

Package: [AP44](README.md). Status: complete; implemented, verified, and independently reviewed.

## Implemented contract

Simple enums require unique resolved integer values, including nonadjacent and
explicit/implicit collisions. FP3044 identifies both variants and the conflicting
value at the later declaration. If any variant has payload fields, every explicit
backing annotation is rejected with FP3045 and a repair preserving the fieldless
alternative. These rules apply to generic declarations and either source order.

Implicit numbering, sequence restarts, integer overflow checks, nominal identity,
type aliases, case coverage and stable sorting retain their existing behavior.
The source grammar still uses unsigned backing literals; the checker also covers
the full signed backing domain represented by the public AST. No new enum runtime
representation, conversion syntax or format version is introduced.

## Implementation ownership

- `crates/fpas-sema/src/check/decl/types/enums.rs` checks the entire declaration
  for payloads and tracks resolved simple values before registering variants.
- `crates/fpas-diagnostics/src/codes.rs` allocates FP3044 and FP3045. Compiler,
  project and editor paths reuse the semantic diagnostics and source spans.
- `crates/fpas-build/build/compiler_identity.rs` includes the changed semantic
  sources in the compiler fingerprint. Source-matching sidecars from a different
  compiler identity are rebuilt and cannot bypass the restrictions. Serialization
  and enum lowering remain unchanged.
- The enum handbook, formal grammar's semantic notes, diagnostic catalog and
  compact reference specify the same contract. Existing repository consumers
  already use valid values; no semantic consumer migration was required.

## Regression ownership

- `crates/fpas-sema/src/tests/decl/enum_values.rs` owns explicit/implicit and
  nonadjacent collisions, equivalent literal spellings, signed AST boundaries,
  whole-data-enum annotation checks, generic declarations, precise locations and
  actionable hints. Existing enum tests retain overflow and restart coverage.
- `crates/fpas-build/tests/incremental/enum_values.rs` executes imported constants,
  type aliases, sorting and case dispatch through cold and reused interfaces. It
  verifies that source-matching legacy compiler sidecars trigger current checks.
- `crates/fpas-language-service/tests/enum_values.rs` checks diagnostic replacement
  and clearing across unsaved edits without changing the on-disk source.
- Existing compiler enum-comparison tests, FPAS sorting tests, CLI examples and
  compact-reference execution retain runtime and authoring coverage. App/example
  projects and distribution precompilation cover source and generated consumers.

## Verification and open follow-ups

Rust formatting and workspace build, AP44 regressions, FPAS regressions,
application/example checks, source formatting, documentation links and release
distribution checks pass. Independent review found no blocking issues.

The workspace runs reproduce the pre-existing VM socket-timeout rounding failure
`write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`
(32 ms observed versus 30 ms expected) and TCP cancellation timing failure
`tcp_backpressure_can_time_out_and_cancel_without_closing`. Their implementations
and tests are unchanged by AP44. VM confirmation excludes those two cases;
workspace doc-tests are checked separately. Baseline evidence is retained in
[AP41's verification notes](../ap41-canonical-qualified-names/02-qualified-name-rules.md).
No selected AP44 work remains open.
