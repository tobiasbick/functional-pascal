# Execution performance review

**Historical review:** the descriptions and source line numbers below refer to
the original revision. See [fixes and verification](fixes.md) for the authorized
implementation follow-up and current evidence.

Revision and review limits: [overview](README.md). The four primary opportunities
below are source-proven unnecessary work or unfavorable scaling, not measured
wall-time bottlenecks. No benchmarks, profiles, changes, or speedup measurements
were produced. Priorities are provisional until representative release measurements
show where time is spent.

## P01 — Text-area navigation and rendering repeatedly scan the document

**Sources:** [Text/TextArea.fpas](../../../lib/Std/Tui/Text/TextArea.fpas), lines
76–103, 177, and 190; [Rendering/TextArea.fpas](../../../lib/Std/Tui/Rendering/TextArea.fpas),
lines 34–46; [Rust CharAt](../../../crates/fpas-std/src/str/mod.rs), lines 186–192.

`TuiTextAreaLocate` repeatedly calls `CharAt(Text, Index)` while finding line
boundaries and counting preceding lines. `CharAt` starts a Unicode-scalar iterator
at the beginning and calls `nth(index)`. A long line with a late caret therefore
causes a sum of prefix scans, with worst-case quadratic work in document length.
The visibility helper can call `Locate` twice; rendering calls it again for the
focused caret.

Rendering also calls `TuiTextAreaLineText` for each visible row. That helper splits
the whole document each time, adding approximately O(visible rows × document size)
work and temporary strings. These costs occur in real editor paths, including Notes.

**Bounded direction:** compute line/caret metrics in one traversal and reuse them
within an input event or frame. Start with existing interfaces and avoid a new
cross-frame cache until invalidation behavior is specified. Preserve Unicode-scalar
caret indexing and terminal display-width behavior; byte indexing is not a substitute.

**Measurement:** long single lines and multiline documents with late carets at
several sizes; test navigation and headless rendering separately. The existing
`unicode_char_at`, `tui_headless`, and `notes_headless` workloads are useful guards
but do not replace a long-document scaling case. Preserve the text-area keyboard,
pointer, layout, and snapshot tests under `tests/stdlib/tui/`.

## P02 — Read-only dictionary operations clone the complete dictionary

**Sources:** [dict.rs](../../../crates/fpas-std/src/dict.rs), lines 15–23 and 41–48;
[intrinsic_args.rs](../../../crates/fpas-std/src/intrinsic_args.rs), lines 155–162.

`pop_dict` always materializes a fresh vector using `pairs.iter().cloned().collect()`.
`Length`, `ContainsKey`, and `Get` use it even though they only read. Consequently,
`Length` does O(n) work and uses temporary O(n) pair storage. Even first-key lookup
copies every key/value before searching. Managed payloads are generally shared,
not recursively deep-copied, but the pair copies and ownership traffic remain.

**Bounded direction:** borrow dictionary storage, analogous to existing
`expect_array`; read the stored length and clone only a returned payload. `Keys`
and `Values` should copy only the selected component. This does not require a
dictionary representation rewrite or a change to insertion-order semantics.

**Measurement:** repeated length and first/middle/missing-key lookups at growing
sizes, with scalar and managed values. The current benchmark manifest has no
dedicated dictionary workload. Add a focused harness case in a follow-up and
preserve absent-key, ordering, and immutability regressions.

## P03 — Array Pop copies the remaining prefix on every call

**Sources:** [compiler calls.rs](../../../crates/fpas-compiler/src/lowering/calls.rs),
lines 195–247; [array.rs](../../../crates/fpas-std/src/array.rs), lines 90–105.

`lower_array_pop` reads the length, retrieves the last element, invokes
`Std.Array.Slice(array, 0, last_index)`, and assigns the result. Slice calls
`to_vec()` on that prefix. Even a uniquely owned local array therefore copies
n−1 values to remove one final element. Draining n elements copies
n(n−1)/2 retained elements overall.

Unlike direct-local `Push`, which emits `Operation::ArrayPush`, this path has no
unique-owner mutation fast path. The comment at the top of `array.rs` referring
to dedicated `ArrayPopLocal` handling does not describe the inspected lowering.

**Bounded direction:** add a consuming/COW pop path for direct locals first,
with explicit ownership and error-order handling. Preserve aliases and the empty
array runtime error documented in [mutating arrays](../../pascal/std/collections/array/mutating.md).
Treat global and captured arrays separately rather than assuming identical ownership.

**Measurement:** drain arrays at several sizes with unique and shared storage;
check returned values and alias preservation. Existing `array_push` does not
measure this path. Add a dedicated stack-drain case before changing implementation.

## P04 — Buffered HTTP repeatedly copies the complete accumulated body

**Sources:** [Http/Client.fpas](../../../lib/Std/Http/Client.fpas), line 266;
[Http/Server.fpas](../../../lib/Std/Http/Server.fpas), line 128;
[array.rs](../../../crates/fpas-std/src/array.rs), lines 107–111;
[intrinsic_args.rs](../../../crates/fpas-std/src/intrinsic_args.rs), lines 170–171.

Each fragment executes `Body := Std.Array.Concat(Body, Bytes)`. Concat copies
both operand vectors before extending. If fragments have fixed size b and there
are k fragments, accumulated-body copying grows as O(b × k²), rather than O(b × k).
Short transport reads increase the number of copies; a requested 65,536-byte read
does not guarantee that many bytes arrive. The buffered client and server body
reader both have this pattern.

**Bounded direction:** accumulate chunks and materialize once, or use a suitable
consuming builder path. Preserve all wire-byte limits, framing validation, error
propagation, and stream close behavior. Do not accidentally make the outer chunk
collection quadratic through another repeated concat.

**Measurement:** identical payloads delivered with controlled small and large
fragments, increasing total sizes, and allocation evidence. Separate transport
latency from accumulation cost. The curated suite currently lacks an HTTP-body
accumulation case. Combine new correctness checks with F02's limit boundaries.

## Secondary candidates: establish exposure before optimizing

### P05 — Generic scalar equality creates a worklist

[value/equal.rs](../../../crates/fpas-bytecode/src/value/equal.rs), lines 4–8,
initializes `vec![(a, b)]` before inspecting scalar variants. Array membership and
dictionary scans repeatedly invoke generic equality. A scalar fast path may avoid
the worklist, but release code generation and allocator evidence must establish
its actual cost. Typed integer comparisons already have a separate path. Preserve
the iterative aggregate traversal, NaN policy, nested values, and dictionary-order
semantics. Do not replace the stack-safe traversal with naive recursion.

### P06 — StoreField introduces an artificial COW owner

[execute/aggregates.rs](../../../crates/fpas-vm/src/vm/execute/aggregates.rs), lines
197–209, clones the destination record before calling `values_mut`, while the old
record is still in its register. [aggregate.rs](../../../crates/fpas-bytecode/src/value/aggregate.rs),
lines 30–35 and 62–65, consequently clones the field vector through `Arc::make_mut`
even for an originally unique destination.

The bytecode selector supports `StoreField`, but ordinary source record updates
in the inspected lowering use `UpdateRecord`. Therefore this is a confirmed local
cost with **unproven representative workload exposure**, not a claimed Notes hot
path. First establish a relevant producer and profile. A later take-before-mutate
change must validate operands first and preserve shared-record isolation. Retain
the shared-record regression in `vm/tests/aggregates.rs`, lines 338–373. Do not
generalize the optimization to `UpdateRecord` without tracing its source aliases.

## Measurement and acceptance procedure

Follow [the project benchmark guide](../../bench/README.md) and
[suite.toml](../../bench/suite.toml), using `cargo bench-fpas`, not Criterion.

1. Select one primary candidate and create any missing representative workload.
2. Save a current release baseline before changing its implementation. For example,
   `cargo bench-fpas save review-before --group tui`; use the same group for comparison.
3. Make one bounded change, preserve regression behavior, and build the release CLI.
4. Compare using `cargo bench-fpas compare review-before --group tui`. Inspect every
   row and repeat measurements when noise prevents a conclusion.
5. Broaden to the full relevant suite for shared VM/value/intrinsic changes. Report
   regressions as well as wins, including workload sizes and allocation evidence.
6. Run the required formatting/build/workspace and relevant FPAS tests. Record only
   a settled measured result in `docs/bench/history.md`.

Complexity improvements alone do not establish an end-to-end speedup. Maintain
separate evidence for steady-state execution, cold/warm compilation, and network
waiting. This review did not rerun the existing historical measurements.

## Existing rewrite decisions remain in force

The current [runtime rewrite options](../performance/runtime-rewrite-options.html)
already retain bounded buffer reuse and defer larger runtime changes. Their
re-entry gate requires profiles attributing at least 25% of target runtime to
costs the proposed rewrite directly removes. Allocation-event percentages are not
CPU-time percentages. None of this review's source traces satisfies that gate.

Do not start a tracing heap, native backend, or new VM encoding from these findings.
The narrow borrowing, scanning, and accumulation opportunities above should first
be measured against the current implementation.
