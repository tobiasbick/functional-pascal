# Query callers plan

Status: planned. Follows [callees](callees.md) and consumes the same compiler
facts; see the [shared implementation plan](README.md).

## Command and target

Proposed invocations after implementation:

```sh
fpas query callers Mandelbrot.fpasproj --file main.fpas --line 5 --column 10

fpas query callers Mandelbrot.fpasproj --file main.fpas --line 13 --column 5 --body --limit 20 --offset 20
```

Run these examples from `examples/mandelbrot`. Reuse callees' context, source/operation anchors, configuration, no-restore,
timeout, owning-project disambiguation and paging validation. Default selection
identifies the checked callable target, constructor or accessor at the position.
`--body` selects an indexed routine/lambda body as a target; initializers/program
entry without a callable target return `not_found/not_callable_target`.
Bodyless interface contracts and external methods remain valid incoming targets.

A declaration-level logical property/event can select its supported accessor
operations; an accessor/constructor use selects the exact checked operation.
Keep logical source declaration and operation identity separate in the result.
Callable storage variables/parameters are not routine declarations: use refs
for their storage uses. Do not convert a variable's callable type into an
invented function target. Missing/unsupported targets retain explicit not_found
reasons, and linked ambiguity lists candidate owners as existing queries do.

## Incoming matching

Index the shared operation catalogue by canonical target identity. Search every
analyzed FPAS project in the explicit context, including checked dependencies.
`--project` chooses the target's owning compilation; it does not narrow the
incoming search. Report actual searched projects separately from result counts.

Match exact overloads, metadata assembly/member identities and generic
definitions. Include generic instantiations under their selected definition,
retaining substitutions on each site. Bridge imported FPAS calls through the
same validated dependency/export mapping as definition/refs; never attach a
binary by a matching display name or signature alone. Linked source files in
different compilations remain different targets and owners.

Return two distinct relationships: actual checked invocations/constructions/
accessor operations, and `callable_reference` uses. The latter can store, return,
pass or subscribe a function without invoking it at that site. Each relationship
retains its operation kind, owning source body and exact site; multiple sites in
one caller remain separate. Direct recursion is a site, not a graph traversal.

An interface/virtual/constrained call matches its bound static contract. Do not
attach it to a guessed implementation, including a single visible candidate.
Delegate invocations with unknown runtime targets cannot be attributed to every
compatible routine or to an earlier assignment without flow proof. Such sites
remain available through callees. Report context-wide unresolved dispatch counts
separately, explicitly without asserting a relationship to the selected target.

This is checked FPAS source scope. Unknown consumers, C#/F# source bodies,
reflection/dynamic calls and metadata-only method implementations are outside the
search. External metadata targets can have known incoming FPAS sites even though
their implementations are unavailable. A complete zero-match answer does not
prove that a routine has no runtime callers or can safely be deleted.

## Result contract

Extend the existing JSON envelope with `result.callers`: selected canonical target
and target mode, indexed/excluded operation policy, context-wide dispatch summary
and a page of incoming sites. Each site has its relationship kind, caller body
descriptor, exact source location, selected static target/operation identity,
dispatch and checked signature/types. Anonymous callers use body kind/location,
not synthesized CLR names or fabricated source declarations.

Expose `totalCount`, `returnedCount`, `offset`, optional `nextOffset` and totals
by actual invocation versus callable reference. Compute all totals before paging;
never label callable-reference counts as an invocation count. Keep any future
caller-body aggregation outside this initial site-oriented result.

Sort by caller project, path, site offset, operation kind and canonical target key.
Preserve repeated sites, distinct same-span operations and linked compilations;
deduplicate only repeated recording of the same semantic operation. Defaults and
limits match callees (100, 1..1000; nonnegative Int32 offset). Truncation concerns
the page, independently of successful input analysis and dispatch uncertainty.

Empty/past-end results are `ok`. Invalid/incomplete project analysis is a failure,
never an apparently complete empty incoming list. Preserve the existing result
size limit, error/cancellation exits and last input verification before output.
Pages require equal snapshot IDs; no durable symbol/body handle is introduced.

## Implementation and proof

- [ ] Index the accepted outgoing facts in Fpas.Workspaces; reuse canonical
  mapping and result descriptors instead of creating a second call resolver.
- [ ] Add typed incoming results and CLI help/JSON with examples that distinguish
  invocation sites, callable references, complete zero and dynamic uncertainty.
- [ ] Prove multi-project/configuration scope, overload/generic separation,
  linked compilation identity and validated source/DLL mapping.
- [ ] Prove direct recursion and repeated calls, references passed/returned/
  subscribed as delegates, lambda callers and unknown delegate invocations.
- [ ] Prove exact static-contract matching for virtual/interface/constrained
  calls; an implementation query must not absorb interface call sites by guess.
- [ ] Prove metadata-only targets with known FPAS callers, excluded foreign or
  unknown consumers, complete-zero versus failed analysis, ordering and paging.
- [ ] Extend existing stale-dependency/changed-input, no-execution and cancellation
  proof at their primary owners. Add transport assertions only for distinct risks.
- [ ] Accept both native Windows and Linux/WSL, including a fresh matching SDK
  and published launcher. Finish the shared regression/documentation matrix and
  remove completed planning material.
