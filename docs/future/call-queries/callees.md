# Query callees plan

Status: planned; first command in the [call-query plan](README.md).

## Command and selectors

Proposed examples after implementation:

```sh
fpas query callees Mandelbrot.fpasproj --file main.fpas --line 5 --column 10

fpas query callees Mandelbrot.fpasproj --file main.fpas --line 13 --column 5 --body --limit 20 --offset 20
```

Run these examples from `examples/mandelbrot`. Require an explicit `.fpasproj`/`.slnx` context, file and positive one-based UTF-16
line/column. Configuration, no-restore, timeout and owning `--project` follow
[existing queries](../../pascal/program-structure/cli.md#query-checked-sources).
Linked files require an unambiguous owner. Project selection disambiguates the
file; it does not replace full-context analysis or dependency validation.

Default selection resolves the callable declaration or bound target at the
identifier/operation anchor, including exact constructor/accessor overloads.
Reuse definition's identity mapping and record any necessary operation anchors
in the new compiler index, including indexer syntax. Do not select a different
routine by name or infer a callable from an arbitrary expression's result type.

`--body` explicitly selects the innermost indexed executable source region
containing the position. It enables lambda, program-entry and initializer bodies
without generated names or new persistent selectors. It can select whitespace
inside such a region; outside indexed regions it returns `not_found/no_body`.
Without this flag, whitespace/comments return `not_found/no_symbol` and
non-callable declarations return `not_found/not_callable_target`. Define exact
operation anchors and half-open region endpoints in compiler/selection tests.

The selected body descriptor states its mode, kind, declaration when present and
source regions. Return both the selection location and canonical target/body;
selecting a call to another routine requests that routine's outgoing operations.
Use `--body` for the operations belonging to the enclosing lexical body instead.

## Outgoing operation semantics

Return one item per owned source operation from the shared
[operation policy](README.md#operation-policy). Keep repeated calls and direct
self-recursion as distinct source sites. Do not recursively expand callees.

Nested routine/lambda operations belong to their own bodies. Creation/conversion
of a callable is a `callable_reference` at the outer site, not evidence that its
body executes there. Calls in requires/ensures are owned by the routine and carry
their phase. Initializers retain their own bodies. A lambda's generated closure
constructor and an async function's generated state-machine methods are excluded.

For `Save(Work)`, report the checked call to Save and the callable reference to
Work. A later delegate invocation records its signature/receiver and an unknown
runtime target; it does not create an inferred call edge to Work. Virtual,
interface and constrained calls retain the statically selected contract and
explicit dispatch uncertainty even when only one implementation is visible.

Source, imported FPAS and metadata-only operations use the same checked identities.
Return exact external identities without loading/executing assemblies. Only
verified FPAS dependency outputs may supply a source body.

## Results and limits

Add `result.callees` to the existing JSON envelope. Preserve context/configuration,
snapshot, diagnostics, coverage, preparation, selection and binary-boundary data.
The result contains selected target/body, body availability, the operation policy,
excluded-category counts, unresolved-target counts and an optional site page.

Available, fully indexed bodies can return `ok` with a complete zero-site page.
A metadata-only callable returns `ok` with availability `metadata_only`; a known
bodyless contract/member says `no_body`. For either unavailable case, omit the
site page/counts instead of claiming that the body has no outgoing calls. Keep
the exact target descriptor available, as inspect already does for metadata.

Each site includes its owner, source extent/anchor, operation kind and phase,
static target when known, dispatch/unknown reason and checked signature/types.
Separate runtime-target uncertainty from source availability. Metadata identity
alone does not make a virtual target statically determined.

`--limit` defaults to 100 (1..1000); `--offset` defaults to 0 (nonnegative Int32).
Sort by project, source path, operation offset, kind and exact target key; preserve
distinct operations at the same position. Order is lexical, not runtime order.
The page carries `totalCount`, `returnedCount`, `offset` and optional `nextOffset`.
Counts/summaries cover the selected body before paging. `truncated` means sites
exist outside the page, including earlier pages. A past-end page is `ok`/empty.
Combine pages only when snapshot identities agree; introduce no persistent cursor.

Validate cheap arguments before preparation. Reuse existing exits, deadlines,
`inputs_changed` rejection and complete 4 MiB `result_limit` failure. Verify inputs
immediately before publishing; never execute the target, its getters or callbacks.

## Implementation and proof

- [ ] Expose typed declaration/body selections through the existing query module.
- [ ] Filter the shared catalogue by exact body ID; preserve same-span operations
  and canonical source/metadata descriptors without re-binding text.
- [ ] Add CLI binding, typed JSON, help/examples and actionable argument errors.
- [ ] Prove routine/member/nested/lambda/entry/initializer ownership and contract
  phases; nested declarations must not inflate the enclosing body's call count.
- [ ] Prove overloads, generics, recursion, construction, accessors/operators,
  source/DLL consistency, callable references and unresolved delegate dispatch.
- [ ] Prove bodyless versus complete-empty results, metadata-only availability,
  operation anchors, Unicode/newlines, linked owners and exact end boundaries.
- [ ] Prove paging, stable ordering, limits and unchanged inputs through compiler,
  workspace and packaged CLI owners. Reuse no-execution/cancellation regressions.
- [ ] Complete CQ2's Windows and WSL acceptance before delivering callers; update
  implemented documentation when the command actually works.
