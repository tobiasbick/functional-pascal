# D5.4: Decide whether whole aggregate history is useful

Status: Deferred; current assessment is to remain deferred. No aggregate
implementation is scheduled. The three scalar-path packages in the
[index](README.md) are implemented and independently reviewed.

## Current recommendation

Keep FPAS 1 tracepoints focused on selected scalar paths. Arrays/records can be
large; retaining their history introduces traversal, comparison and memory costs
which the scalar delivery deliberately avoids. D1/D3 already provide bounded
aggregate snapshots for inspection.

Use concrete debugging cases to determine whether selecting leaf paths misses
important information. A vague wish to trace everything is not enough reason
to add deep history, a second result mode or new language/runtime machinery.

## Recorded assessment

The current delivery provides scalar record/array paths, explicit absent positions
and activation-scoped captures. No two concrete debugging cases requiring deep
aggregate history have been established during this implementation. This is not
a claim of exhaustive dogfooding: there is no evidence to justify aggregate code
or larger retention limits now. Keep using bounded D1/D3 aggregate snapshots at
the useful stop. Whole aggregates and continuous dormant escaped-cell history
remain deferred; neither is a remaining implementation commitment.

Reopen the decision with concrete evidence using the checklist below. Deferral
completes this delivery's assessment, not whole-aggregate support.

## Reopening criteria

- [ ] Record at least two real cases which packages 1–3 cannot explain; include
  the existing trace/snapshot output and the specific missing information.
- [ ] Compare smaller remedies: a length/count scalar in existing program state,
  additional selected paths, or a better D1/D3 snapshot at the useful stop.
- [x] Current decision: **remain deferred**, for the reason above. If the two
  concrete cases justify implementation later, write a separate bounded subset
  package before adding code.

If implementation is justified, write a separate concrete implementation package
before adding code. That package must settle:

- Supported value kinds and exact structural equality, including dictionary
  key/order semantics, aliasing, nominal types and known unavailable children.
- Detached snapshots, per-event/live/total node and byte limits, depth limits,
  traversal/comparison work and behavior on the first exceeded bound.
- A complete comparison input versus clipped presentation: equal displayed
  prefixes, shallow summaries or unchecked hashes must never imply equality.
- Initialization, root replacement, affected-watch ordering, lexical/capture
  lifetimes, delayed results, report clipping and explicit coverage/counts.
- How to avoid retaining live program graphs, cells, handles or observer-created
  cycles. Unsupported resources remain unsupported rather than executing getters.
- VM/CLI/image tests, documentation, independent subagent review and local
  performance measurements where meaningful. Cloud timings are not acceptance data.

Do not extend the current limits silently, claim deep history from truncated
snapshots or implement D2 record/replay as part of this decision.
