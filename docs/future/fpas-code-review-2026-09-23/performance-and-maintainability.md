# Performance and maintainability

These observations come from source inspection. No benchmark was run, and no speedup is claimed. Correctness fixes in the companion report should precede broad optimization work.

## P01, P2: UTF-8 conversion performs quadratic work

Source: `lib/Std/Net/Utf8.fpas:10` and `:66`, especially lines 15 and 148.

Encoding reads character `Index` through `Std.Str.CharAt` on every iteration. Its intrinsic implementation in `crates/fpas-std/src/str/mod.rs:191` uses `chars().nth(index)`, traversing the string prefix again. Summed over the input, this is quadratic character traversal.

Decoding appends one scalar at a time with `Text := Text + Chr(CodePoint)`. String addition reaches `SharedStr::concat` through `crates/fpas-vm/src/vm/value_ops/scalar.rs:57`. The implementation at `crates/fpas-bytecode/src/value/string.rs:75` copies both strings into new storage. This gives quadratic output copying, including for ASCII.

These are general network codecs, so larger HTTP and AI payloads inherit the cost. Prefer a bulk codec or a single-pass traversal with an output builder using existing runtime facilities. Preserve strict rejection of invalid encodings. Measure doubling input sizes with ASCII and multibyte text before selecting the implementation. Keep short correctness cases, and add malformed sequence boundaries independently of performance measurements.

## P02, P3: TUI queue draining copies each remaining suffix

Source: `lib/Std/Tui/Runtime/Application/State.fpas:106`, `:115`, `:126`, and `:131`.

Both pop functions replace their arrays with `Slice(queue, 1, length - 1)`. Draining a burst of n queued items therefore copies successively shorter arrays. Enqueue also uses concatenation. This matters for synthetic input bursts, accumulated messages, and headless application tests more than ordinary single key presses.

Store a read index and compact occasionally, or use an existing queue representation. Preserve FIFO ordering, empty-queue behavior, and isolation between application handles. Measure burst enqueue/drain work before changing the representation.

## P03, P3: Text area navigation repeatedly splits text and measures prefixes

Source: `lib/Std/Tui/Text/TextArea.fpas:22`, `:32`, `:43`, `:70`, `:104`, and `:129`.

Line count, line lookup, line start, caret lookup, and content sizing independently split the full text. `TuiTextAreaCaretAtDisplayColumn` then builds and measures every prefix of a line until it reaches the target column. On long lines, total prefix work grows quadratically. Large unwrapped notes and pasted content can exercise this path.

Compute line boundaries once per operation or text revision. Use one traversal for display-column lookup while preserving the console's grapheme and cell-width rules. Do not replace it with scalar-count arithmetic. Verify combining characters, wide characters, empty final lines, and caret positions beyond the visible viewport. Benchmark long single lines separately from many short lines.

## P04, P3: Every Notes resort uses pairwise comparisons

Source: `apps/notes/src/Notes/Repository.fpas:45` and `apps/notes/src/Notes/Update.fpas:37`.

`NotesSort` compares every pair through nested loops. Loading calls it, and a successful save calls it again through `ResortSelected`. The work is quadratic in note count even when only one note's timestamp changed.

Use an existing comparison sort if available, or update the selected note's position using the same ordering rules. First measure realistic directory sizes. Keep pin ordering, timestamp ordering, and selected identity stable; fix duplicate identity handling in F03 before relying on identity-based repositioning.

## M01, P3: Two fractal explorers duplicate the interaction loop

Source: `examples/math/burning_ship/burning_ship.fpas` and `examples/math/tricorn/tricorn.fpas`, each about 350 lines.

Both define the same sequence of viewport conversion, HUD writing, redraw, key handling, mouse handling, resize handling, event dispatch, and terminal shutdown functions. Representative matching locations are `ViewportCX` at line 73, `HandleKey` at line 162, `HandleMouse` at line 252, and `ShutdownTerminal` at line 310.

Terminal handling fixes require editing both copies. If these remain full interactive explorers, extract their shared camera and terminal lifecycle into a focused example support project while retaining separate fractal renderers and palettes. If standalone educational readability is the priority, keep small standalone demonstrations and reduce the duplicated application shell. Do not introduce a general UI framework solely for these examples.

## M02, P3: Notes spreads action and control numbers across units

Source: `apps/notes/src/Notes/Update.fpas:7`, `apps/notes/src/Notes/Editing.fpas:6`, and control construction in `apps/notes/src/Notes/View.fpas`.

Update names control and action numbers, Editing repeats a subset, and View supplies corresponding numeric values to TUI builders. Tests also construct numeric actions. The compiler cannot detect disagreement between these copies.

Put these identifiers in a focused `Notes.Actions` or `Notes.Controls` unit and use the names at producers and consumers. Retain a behavioral test that sends an event from the actual view, so sharing constants does not make tests merely repeat implementation choices. This is a maintenance finding; no current numeric mismatch was found.

## M03, P3: HTTP setup obscures straightforward error propagation

Source: `lib/Std/Http/Client.fpas:50`, before `OpenConnection`.

`OpenOnce` creates an empty URI record and an empty encoded buffer, then assigns each in a result match whose error branch only returns that error. These temporary invalid values and mutable declarations can be replaced by typed `try` bindings. The existing language and application code already support this idiom.

Limit that simplification to operations before connection ownership begins. Later error branches close a live connection and must retain that cleanup. URI validation, framing checks, handle checks, and configured resource limits are necessary defensive programming; removing them would weaken contracts. The unnecessary part here is the propagation boilerplate, not the validation itself.

## Reviewed limitations that are not new defects

Local Chat's synchronous request and omission of conversation history are described by its current README and service implementation. Its visible transcript is not sent as context. Changing this requires product work rather than treating the existing behavior as an undocumented regression.

Generated `lib/api/Std` declarations and intentional intrinsic entry points should be changed through their owning metadata/runtime path. Their placeholder implementations should not be rewritten as ordinary FPAS functions during cleanup.
