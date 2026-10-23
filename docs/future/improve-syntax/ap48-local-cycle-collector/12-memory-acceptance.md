# AP48.12: Memory correctness, performance and package acceptance

Package: [AP48](README.md). Status: planned.
Prerequisite: [AP48.11](11-runtime-replacement.md) integrated replacement.
Effort: large. Decision: Q49.

## Acceptance objective

Demonstrate complete lifetime behavior, bounded retained memory after work ends,
and measured execution/transfer costs for the final runtime. Tests cannot prove
the absence of every memory bug; combine observable lifetime tests with an audit
of graph edges, roots, ownership transitions and unsafe code.

The original AP48.1 through AP48.7 remain delivered work. The extended AP48
package is complete only after AP48.8 through AP48.12 meet their own criteria.
Do not label a source review, prototype or successful microbenchmark as completion.

## Memory test matrix

- Acyclic and cyclic graphs, self-recursive and mutually recursive closures,
  aggregates hiding captures, shared subgraphs and deep graphs.
- A rooted graph survives collections with unchanged identity/state; dropping one
  of several independent roots releases only the now-unreachable portion.
- Routine returns remove stack roots without destroying escaping closures.
  Local aliases retain their selected semantics after frame/register growth.
- Allocation stress collects at every legal constructor/call boundary. Temporary
  values, partially constructed objects and caller roots survive nested callbacks.
- Task success, failure, cancellation, structured scopes, supervision retries,
  worker migration and suspended/native-blocked execution release appropriate roots.
- Channel queues, selected and abandoned cases, task results and failed transfers
  neither leak graphs nor publish partially initialized objects.
- Host/debugger handles survive task or session retirement according to the new
  contract; last and non-final releases trigger required maintenance.
- Concurrent handle access/release and collection cannot lose requests, overlap
  exclusive access, deadlock, resurrect invalid objects or use stale identities.
- Native resources close exactly as their contract requires. Managed GC cycles
  cannot retain hidden native-to-managed cycles indefinitely.
- Destroying one VM/domain does not invalidate another domain's valid shared
  resources or executable metadata; unrelated tasks progress during local GC.

Reuse existing tests where their observable contract still applies. Port the
access-guard regressions to equivalent APIs rather than deleting their protection.
Add cases for new boundaries, not tests that merely duplicate collector code.

## Measuring reclamation

Warm up runtime caches, record a controlled baseline, create/drop a bounded graph,
force the appropriate collection/retirement maintenance, then compare independent
allocation/deallocation and live-payload counters. Account separately for heap
capacity/free lists, code caches, external resources and shared immutable storage.
The empty logical heap need not return every cached page to the OS immediately.

Repeat churn in a persistent VM and with retained handles. With a fixed live set,
live objects and payload bytes must return to the explained baseline and retained
capacity must stay within the documented allocation policy. Include large
variable-size payloads so object-count balance cannot hide leaked buffers.
Measure peak memory as well as post-collection memory. Use deterministic lifetime
witnesses and an independent allocation observer where practical; the collector's
own counters alone are insufficient evidence.

Run race-focused stress and targeted Miri or sanitizers where supported and useful
for the selected implementation. Record unsupported coverage honestly. RSS is an
additional observation, not the pass/fail oracle for object reclamation.

## Performance comparison

Use the AP48.10 protocol against the corrected old baseline and the fully
integrated candidate. Store all benchmark artifacts under
`.temp-data/ap48-memory-acceptance/`. Keep cloud comparisons within that environment;
do not compare them to unrelated historical machines or rewrite tracked timing
history. Record exact source/build identities and collector configurations.

Include capture-heavy workloads, array callbacks, calls/loops without captures,
aggregate copying, channel/task transfer, large live sets and repeated cycles.
Report execution times and variation, transfer/copy costs, allocation volume,
peak/live memory, collection count and pause distribution. Validate program
outputs first. State the percentage formula and whether samples are paired.

Report regressions as well as improvements. The decision can favor a simpler
correct implementation, but any material speed/memory tradeoff must be explicit.
There is no pre-approved speedup percentage, pause bound or permission to weaken
correctness for a benchmark. If adoption fails, record a revise/abandon decision
and the remaining package status instead of marking acceptance complete.

## Final verification and delivery

Run the repository's applicable checks on the final integrated tree: Rust
format/build/workspace tests, strict workspace Clippy, the FPAS suite and affected
example/app checks. Run formatter/grammar/editor checks if syntax or fixtures
changed. Validate documentation links and `git diff --check`. Do not waive a
failure merely because it appeared before this work; fix it or record a concrete
blocker that prevents completion. Distinguish environment limitations from product
failures with evidence.

Review the final design and implementation against the ownership matrix, root
inventory, native cleanup and concurrency contracts. Arrange independent review
when authorized; this planning task itself does not start agents or implementation.
Fix confirmed findings, repeat affected checks and preserve any coverage limits.

Remove temporary bridges and obsolete APIs/options. Update current manuals and
runtime inventories, mark work packages complete only with evidence, then update
the central package status. Record the final chosen architecture, remaining
tradeoffs and measured results without retaining obsolete design alternatives as
production requirements. Commit/push/merge follow the user's authorization for
that implementation work; this document does not pre-authorize a later merge.
