# FPAS correctness review

**Historical review:** the descriptions and source line numbers below refer to
the original revision. See [fixes and verification](fixes.md) for the authorized
implementation follow-up and current evidence.

Revision and verification limits: [review overview](README.md). These three P2
findings are open and have high-confidence source traces. Proposed regressions
were not executed. FPAS execution costs are in [the performance report](performance-review.md).

## F01 — Cursor movement marks Notes content dirty

**Location:** [Notes/Update.fpas](../../../apps/notes/src/Notes/Update.fpas), lines
430–450 and 457–467.

Title, tags, and body update handlers unconditionally set `Dirty := true` and
`Status := 'Unsaved changes'`. The TUI also emits these messages for caret-only
changes: [Routing/Input.fpas](../../../lib/Std/Tui/Runtime/Routing/Input.fpas),
lines 185–206, and [Routing/TextArea.fpas](../../../lib/Std/Tui/Runtime/Routing/TextArea.fpas),
lines 130–138 and 163, can retain exactly the same text while changing the caret
or viewport.

**Trigger:** load a clean existing note and press an arrow/Home/End key in its
title or body, or place the body caret with the pointer.

**Effect:** navigation is treated as a persisted content edit. It can produce a
quit confirmation and unnecessary saving on selection changes. Persistence
updates `UpdatedMillis` and sorts the notes (`Update.fpas`, lines 92–105), so
navigation can also change modification metadata and ordering.

**Contract:** [text-area controlled updates](../../pascal/std/tui/text-area.md)
carry text, caret, and viewport proposals together; receiving the message does
not itself establish that text changed. Notes already uses `Dirty` as the guard
for actual persistence.

**Fix direction:** always accept caret/offset changes, but mark dirty only when
persisted content changes. Preserve an already-dirty state. Handle tags according
to the application's persisted representation rather than assuming every proposal
is an edit.

**Regression:** a clean loaded note stays clean after title/body navigation and
pointer placement; actual text changes become dirty; navigation on an already
dirty note does not clear it. Verify no navigation-only save/timestamp change.
[note_tui_workflow_test.fpas](../../../tests/apps/notes/note_tui_workflow_test.fpas),
lines 151–157, tests changed content but not this distinction.

## F02 — Exact-limit close-delimited response is rejected

**Location:** [Http/Stream.fpas](../../../lib/Std/Http/Stream.fpas), lines 107–111
and 236–251.

`ReadNetwork` returns a limit error as soon as `BytesReceived = Maximum`, before
trying to observe EOF. `ReadUntilClose` calls it after the buffered bytes have
been consumed because connection-delimited bodies need EOF to establish completion.

**Trigger:** a response without `Content-Length` or `Transfer-Encoding`, with a
total header-plus-body byte count exactly equal to `MaxResponseBytes`, followed
by a normal peer close.

**Effect:** the next stream read fails instead of reporting EOF. Since `Send`
drains the same reader, it returns an error instead of the complete response.
The presence of a normal close cannot help: the guard runs before the socket read.

**Contract:** [HTTP limits](../../pascal/std/network/http.md), lines 108–113,
bounds all received wire bytes, including headers and framing. Exactly the maximum
does not exceed that bound. Content-length-delimited success does not cover this
separate EOF path.

**Fix direction:** distinguish EOF at the boundary from an actual extra byte.
Any bounded probe must reject additional input without silently accepting a
truncated body, and preserve existing timeout/close behavior.

**Regression:** serve raw close-delimited responses with total wire sizes N−1,
N, and N+1 against limit N. Check streaming and buffered APIs; include a header-only
response with an empty body. Count header bytes in the fixture. No matching exact
response-limit regression was found in the inspected network tests.

## F03 — SSE finalization charges synthetic delimiters to the input limit

**Location:** [Http/Sse.fpas](../../../lib/Std/Http/Sse.fpas), line 261 and lines
113–116.

`Finish` calls `Feed(Decoder, [10, 10])`. `Process` includes those artificial line
delimiters in `EventBytes` and applies the normal input limit.

**Concrete state trace:** create a decoder with `MaxEventBytes = 7`, then feed
the seven UTF-8 bytes of `data: x` without a newline. Feeding succeeds because
the buffered input fits. Finalization adds a newline, counts eight bytes, and
returns `SSE event exceeds MaxEventBytes` instead of dispatching data `x`.

**Contract:** [SSE behavior](../../pascal/std/network/http.md), lines 120–121,
bounds buffered input for an event and promises final-event dispatch by
`FinishSse`. The extra bytes were not input from the caller.

**Fix direction:** implement final-line/event processing explicitly, without
charging synthetic input. Preserve the finished-state contract and the existing
handling of fragmented UTF-8 and CR/LF sequences.

**Regression:** finish `data: x` with limits 6, 7, 8, and 9; distinguish genuine
oversize input from finalization overhead. Add coverage for already-terminated
events, empty input, final CR, and rejected input after successful finalization.
An empty decoder with limit 1 is not itself a demonstrated failure: each blank
line resets `EventBytes`.

[http_streaming_test.fpas](../../../tests/stdlib/net/http_streaming_test.fpas)
tests finalization with a generous limit and oversized input separately, leaving
the exact-boundary combination uncovered in the inspected cases.
