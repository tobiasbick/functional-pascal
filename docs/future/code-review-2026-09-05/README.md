# Rust and FPAS code review — 5 September 2026

Reviewed revision: `1648b9f3fe4731d2031d0f2ebde454269e0d2c35`.
The working tree was clean when inspection started. This is a review of the
current implementation, not a review limited to the latest commit.

## Reports

Current implementation status: [fixes and verification](fixes.md).


- [Rust correctness review](rust-review.md): compiler, numeric conversion, and network cancellation.
- [FPAS correctness review](fpas-review.md): Notes state changes and HTTP/SSE boundaries.
- [Execution performance review](performance-review.md): concrete cost paths, measurement priorities, and acceptance criteria.

## Original correctness priorities

All six findings are P2: actionable defects under specific inputs or workflows.
Within that severity, investigate cancellation first because it can delay shutdown,
then silent numeric conversion, compilation failure, protocol boundaries, and Notes.
Confidence describes source evidence, not a successful runtime reproduction.

| ID | Finding | Evidence | Report |
| --- | --- | --- | --- |
| R03 | Outgoing TLS handshake is outside cancellation tracking | High-confidence lifecycle trace; timing reproduction pending | [Rust](rust-review.md#r03--outgoing-tls-handshake-is-not-interrupted-by-vm-cancellation) |
| R02 | Real-to-integer conversion accepts positive 2^63 | Direct boundary/data-flow proof | [Rust](rust-review.md#r02--real-to-integer-conversion-accepts-positive-263) |
| R01 | Closure discovery skips the repeat-until condition | Direct traversal omission | [Rust](rust-review.md#r01--closure-discovery-skips-the-repeat-until-condition) |
| F02 | Exact-limit close-delimited HTTP response is rejected | Direct EOF/limit trace | [FPAS](fpas-review.md#f02--exact-limit-close-delimited-response-is-rejected) |
| F03 | SSE finalization charges synthetic delimiters to the input limit | Direct decoder-state trace | [FPAS](fpas-review.md#f03--sse-finalization-charges-synthetic-delimiters-to-the-input-limit) |
| F01 | Cursor movement marks Notes content dirty | Routing-to-update trace | [FPAS](fpas-review.md#f01--cursor-movement-marks-notes-content-dirty) |

## Original performance priorities

1. **P01: Text-area scans.** Repeated character indexing and full-document splitting affect interactive editing.
2. **P02: Dictionary reads.** Even `Length` copies every pair; a narrow borrowing change has a clear complexity target.
3. **P03: Array Pop.** Draining a local stack copies every remaining prefix.
4. **P04: HTTP body accumulation.** Fragmented input repeatedly copies the complete accumulated body.

This ordering balances exposure and scope; it is not a measured ranking of CPU
time. The performance report also identifies two lower-confidence workload
candidates. No speedup percentage is claimed, and no rewrite is authorized.

## Original scope and method

The tracked source inventory contains 1,397 Rust files and 728 FPAS files. This
is a risk-focused review, not a line-by-line certification of all those files.
Independent runtime and FPAS/spec inspections were cross-checked during aggregation.

Detailed inspection covered compiler closure discovery, expression/control-flow
and collection lowering; VM aggregate operations, shared values, intrinsic argument
handling, callbacks, tasks and network shutdown; Rust array/dictionary/math/string
implementations; FPAS HTTP client/server/framing/stream/SSE; TUI text-area rendering
and input routing; Notes persistence/update; and selected Local Chat paths.
Build/project/linker structure, benchmark coverage, and existing performance plans
were sampled for context.

Contracts and counterexamples were checked against relevant `docs/pascal/` pages,
existing Rust tests, and FPAS tests. Findings include proposed regression scenarios;
those scenarios were **not executed**. No `cargo build`, `cargo test`, FPAS execution,
Clippy, benchmark, or profiler run was performed. There are no fresh baseline
timings, allocator counts, or passing-test claims in these reports.

The lexer/parser, semantic analyzer as a whole, debugger, LSP/language service,
formatter, CLI packaging, artifact verifier, and all platform-specific backends
were not exhaustively reviewed. Example programs and tests were sampled, not all
executed. Absence of a finding in those areas is not evidence of correctness.

## Standards and specification

**Standards axis:** ownership/copying concerns with concrete execution consequences
are in the performance report. No additional naming, file-size, or style-only
finding is promoted to a defect. Tool-enforced formatting/lint rules were not audited.

**Specification axis:** the correctness reports link implementation evidence and
the relevant current contract. Existing repeat-body scope rules and Local Chat's
documented synchronous request behavior were not treated as bugs. A suspected
empty-SSE-finalization failure was rejected: blank-line processing resets the event
counter, so it does not demonstrate F03.

## Implementation follow-up

The subsequent request authorized fixing every listed item and making a local
co-authored commit. The original review below and in the three reports records
findings at the reviewed revision; its static-only limitations apply to that
inspection, not to the implementation follow-up.

See [fixes and verification](fixes.md) for the changes, regression evidence,
measurement results, and completed validation. No language syntax or semantics
were redesigned. No push is requested.
