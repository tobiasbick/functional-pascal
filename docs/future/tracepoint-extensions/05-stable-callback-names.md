# D5.5: Stable executable names for anonymous callbacks

Status: Planned; next debugger implementation package. No implementation is
part of the Notes application fix. See the [index](README.md).

## Problem and selected contract

Anonymous callable names currently contain the AST expression's address, such
as `Outer.$closure_140...`. That key is valid inside one immutable compilation,
but changes between processes. Exact `Routine::Binding` trace requests obtained
from one compilation cannot reliably be reused for another source compilation.

- Keep one exact trace-selector syntax. Do not add callable-variable aliases,
  suffix searches, a second selector mode or runtime name guessing.
- Name anonymous source callbacks deterministically within their lexical owner:
  `Owner.$closure_1`, `Owner.$closure_2`, and so on, in lexical source order.
  Number independently within each owner, starting at 1. Nested anonymous and
  named routines keep the complete owner path, including unit/specialization
  components required to distinguish executable routines.
- Identical sources and compilation inputs must produce identical names across
  separate processes, source/project targets and object/image round trips.
  Names may change when earlier callbacks are inserted or source ownership
  changes. This is not a persistent identity across edits.
- Preserve address-based AST keys internally where they identify the same
  immutable node. Separate those lookup keys from published routine names.
- Never merge closures, captures or activations because names or values match.
  Existing function IDs, capture provenance and activation lifetimes remain.
- Audit generated callable adapters for the same address leak. Cover affected
  user-visible names with the same deterministic ownership rule; record any
  genuinely separate design requirement rather than silently widening the task.

## Work and acceptance

- [ ] Separate deterministic source naming from semantic expression lookup keys.
- [ ] Preserve nested owner names, distinct same-named units, applied generic
  routines, retained defaults and named helpers inside anonymous functions.
- [ ] Verify two separate compiler processes produce the same published names;
  do not prove stability only by compiling twice in one live AST/session.
- [ ] Obtain a callback name from a batch stop, then use it in a fresh source
  compilation's `--trace` request and prove the intended capture is selected.
- [ ] Cover independent/copy/shared captures, shadowing, different closure
  literals in one owner and source/project/object/portable-image targets.
- [ ] Run affected semantic/compiler/debugger/CLI regressions and strict checks.
- [ ] Obtain independent review, fix findings, update current docs and wishlist,
  then remove this completed temporary package.

This package changes generated names, not source syntax or closure storage.
It does not introduce unrestricted escaped-cell history, aggregate tracing,
record/replay or a performance acceptance claim from cloud timings.
