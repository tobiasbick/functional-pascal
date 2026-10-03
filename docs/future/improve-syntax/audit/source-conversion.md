# Stage 3 source conversion verification

Scope: the source-conversion item in [stage 3](../stages/03-syntax-and-names.md),
limited to the implemented [block/name](block-syntax-delivery.md) and
[operator/bit API](operator-delivery.md) deliveries. The conversion already
landed with those deliveries; this verification closes the remaining checklist
item and adds missing integration coverage. No additional source rewrite or
conversion tool is required for these implemented constructs.

Conditional/case expressions belong to stage 4, and task scopes belong to stage
5. Their source migration is part of those owning implementations. Existing
methods, properties, events, and `mutable` declarations remain until their
coordinated replacement; they are not stage-3 conversion failures.

## Migrated consumers

The block/name delivery covers explicit aliases, qualified references, named
closers, individual declarations, terminators, branch scopes, and `NullValue`.
The operator delivery inspected the previous syntax and semantic metadata,
preserved explicit grouping, and replaced integer operators with imported bit
functions. Short-circuit skipping follows the agreed new evaluation rule;
`xor` remains eager. Their delivery records include Rust/TypeScript fixtures,
bench source generators, intrinsic API declarations, scaffolds, and editor
consumers. Temporary conversion tools and a legacy parser mode are absent from
the delivered implementation.

The current repository corpus contains 135 library/API files, 129 example
files, 541 test files, and 17 app files. All four trees pass canonical formatting
and structural round trips. The new library-tree regression includes both
`lib/Std/` source units and `lib/api/Std/` generated intrinsic declarations.
The existing round-trip assertion checks the normalized AST, every comment,
successful reparsing, and identical second formatting.

All 22 example/app project manifests pass `fpas check`. Of 109 example/app
programs, 95 pass standalone checks; the other 14 report missing project units
when checked without their manifest and are covered by the successful project
checks. The trusted `lib/stdlib.fpasprj` is loaded through the standard-library
path, not checked as a user project: the user-project loader intentionally
rejects the reserved `Std` root. Library syntax is checked by the new corpus
test; standard-library loading and execution remain covered by workspace and
FPAS suite tests.

## Added regression coverage

`crates/fpas-cli/src/main_tests/fmt/source_conversion.rs` uses the actual CLI
dispatch, project loader, compiler, linker, and VM with existing test support.
It adds four tests:

- Execute a migrated program before and after in-place formatting. Exact
  output proves nested `else if` ownership, explicit-block and case-arm scopes,
  outer-binding preservation, anonymous argument boundaries, written bit-call
  argument order, short-circuit skipping, eager `xor`, and comparison precedence.
  Comments and strings containing obsolete spellings remain intact; the result
  passes `fmt --check`.
- Execute and format a project whose two imported units export identically
  named types, variants, and routines alongside a local routine of that name.
  Case-insensitive aliases retain the resolved owners before and after source
  formatting and compiled-unit rebuilding.
- Reject mixed logical chains, chained comparisons with a callable middle
  operand, a missing nested closer, an alias-free import, a legacy program
  closer, an infix shift, and missing/extra terminators. Each diagnostic retains
  its replacement guidance; formatting leaves the rejected file byte-for-byte
  unchanged and emits no source output.
- Reject unqualified/full-unit access, nested case-insensitive alias shadowing,
  and a reference into a neighboring branch's scope through semantic `check`.
  Checking leaves the source unchanged. Formatting does not attempt semantic
  owner inference or invent aliases.

`crates/fpas-fmt/tests/round_trip.rs` adds the fifth test: full library and
generated-API structural round trips, using the existing shared assertion.
Existing parser, semantic, compiler, formatter, real-process, and editor tests
cover the owning language rules; the additions check their migration
interactions without duplicating the entire grammar matrix.

## Verification

- `cargo fmt` and `cargo build --quiet` passed.
- `cargo test --workspace --quiet` passed, including the new CLI tests, all
  corpus round trips, existing language tests, real-process tests, and LSP
  formatting tests.
- Targeted CLI source-conversion tests: four passed. Targeted formatter
  round-trip suite: five passed, including the new full library/API test.
- `fpas fmt --check lib examples tests apps` passed for all 822 source files.
- `fpas test tests/suite.fpasprj --jobs 8`: 459 passed, one intentionally
  skipped, zero failed.
- All 22 example/app project checks passed; the 109 standalone program checks
  produced the 95 standalone and 14 project-dependent results described above.
- `node editors/vscode/scripts/verify-grammar.mjs` passed.
- All 47 relative file links in the changed planning documents resolve.
- `git diff --check` passed.

## Plan reconciliation

The source-conversion checkbox now names its implemented constructs and links
to this evidence. The central stage index, previous delivery's remaining-work
paragraph, and implementation sequence reflect completed operator and migration
work. The [coordinated documentation delivery](documentation-delivery.md)
closes the documentation item. The [full coverage delivery](coverage-delivery.md)
verifies the final checklist item and closes stage 3 for implemented constructs.

Current-language documentation and Rust documentation links are unchanged:
this delivery adds verification of implemented behavior, without changing
language syntax, semantics, public APIs, or formatter output.
