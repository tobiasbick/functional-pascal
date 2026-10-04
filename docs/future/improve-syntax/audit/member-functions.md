# Ordinary functions replacing record members

Status: source, implementation, editor and documentation migration is complete.
Final workspace and consumer verification passes; the stage-4 member-removal
and documentation acceptance items are complete.

## Verified source migration

Standard units, applications, examples and FPAS regressions now use ordinary
functions, explicit record arguments, snapshot closures and optional handlers.
The complete FPAS suite passes with 470 tests and one intentional skip. All 22
application/example projects pass; the loose-source checks cover 95 of 109
programs, with the remaining 14 requiring their verified project contexts.

The constructor-only builder records are removed. Eight public TUI facade units
expose ordinary functions without exporting their internal implementation units.
Private fields and routines belonging to private record types retain their
visibility. Examples and regression files have names matching their replacements.

Embedded compiler fixtures preserve argument order, early `try` exits, record
snapshots, generic calls and optional handlers. The compiler suite passes with
248 tests. Five controlled debugger-call tests and eleven callable-assignment
tests pass with ordinary functions and explicit closures. All 120 CLI project
tests pass, including private factories, imported record values and cold/warm
purity/default checks. Fourteen generic semantic tests pass with explicit record
parameters. These are migration checkpoints, not final stage acceptance.

## Replacement layout

Move each routine out of its record and keep it in the declaring unit. Prefix
its name with the record name, removing the redundant `Builders` suffix for
constructor-only namespaces. Existing explicit receiver parameters become
ordinary positional parameters. Keep field and helper visibility unchanged.

Modify these declaring sources:

- `lib/Std/Ai/OpenAi/Types.fpas` - move record routines to unit scope.
- `lib/Std/Http/Types.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/ButtonPalette.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/Cell.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/Color.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/InputPalette.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/Palette.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Cells/Style.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/Button.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/CellGrid.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/Gauge.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/Input.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/KeyGesture.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/MenuItem.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/MenuNode.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Chrome/StatusItem.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Elements/Element.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Geometry/Point.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Geometry/Rect.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Geometry/Size.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Ids/Action.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Ids/ControlId.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/Alignment.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/AlignmentBounds.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/Frame.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/LayoutFit.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/LayoutSettings.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/Margins.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/MeasureAxis.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/MeasureConstraint.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/MeasureResult.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/MeasureSpec.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/SizePolicy.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Layout/Spacer.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Rendering/Canvas.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Rendering/Surface.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Rendering/Surface/Snapshot.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Runtime/Application.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Runtime/Cmd.fpas` - move record routines to unit scope.
- `lib/Std/Tui/Runtime/PointerEvent.fpas` - move record routines to unit scope.

Create the following exported facades. Each delegates to the declaring units;
internal units and non-public fields remain internal.

- `lib/Std/Tui/Geometry.fpas` - public geometry functions.
- `lib/Std/Tui/Cells.fpas` - public cells functions.
- `lib/Std/Tui/Chrome.fpas` - public chrome functions.
- `lib/Std/Tui/Elements.fpas` - public elements functions.
- `lib/Std/Tui/Layout.fpas` - public layout functions.
- `lib/Std/Tui/Rendering.fpas` - public rendering functions.
- `lib/Std/Tui/Runtime.fpas` - public runtime functions.
- `lib/Std/Tui/Ids.fpas` - public ids functions.

Modify `lib/Std/Http.fpas` and `lib/Std/Ai/OpenAi.fpas` to expose their ordinary
constructor functions, and `lib/stdlib.fpasprj` to export the new TUI facades.
Remove empty builder records and their aliases in `lib/Std/Tui.fpas` once all
callers use the replacement functions.

Migrate resolved calls under `lib/`, `apps/`, `examples/` and `tests/`, then
Rust-embedded sources, handbook programs and editor fixtures. Rename the obsolete
member examples to describe ordinary functions, closures and optional handlers.
Retain explicit negative cases for removed syntax. Update the TUI, HTTP and
OpenAI API pages with the implemented function names and import qualifiers.

## Checks before deleting implementation paths

- Resolve each target using semantic metadata, including imported aliases.
- Preserve receiver-before-argument evaluation, written argument order, private
  construction, generic inference and actual mutation behavior.
- Replace bound methods with explicit closures over a receiver snapshot captured
  once when the value is created.
- Replace property access with ordinary getter/setter calls. Replace events with
  optional handler storage and explicit exhaustive matching.
- Verify consumers with the existing compiler before removing obsolete parser,
  semantic, compiler, runtime, formatter and editor support.
- Complete the shared stage verification after all implementation and documentation
  paths have been migrated.

## Application and example layout

Move routines and remove property declarations in the following sources. Preserve
source-unit ownership, private fields and existing observable results:

- `apps/local-chat/src/LocalChat/Model.fpas`
- `examples/math/burning_ship/burning_ship_render.fpas`
- `examples/math/explorer/Camera.fpas`
- `examples/math/julia/julia.fpas`
- `examples/math/julia/julia_compute.fpas`
- `examples/math/mandelbrot/mandelbrot_model.fpas`
- `examples/math/mandelbrot/mandelbrot_render.fpas`
- `examples/math/newton/newton.fpas`
- `examples/math/newton/newton_render.fpas`
- `examples/math/tricorn/tricorn_render.fpas`
- `examples/pascal/functions/postfix_chaining.fpas`
- `examples/pascal/generics/generic_record_methods.fpas`
- `examples/pascal/record-methods/bound_methods.fpas`
- `examples/pascal/record-methods/counter.fpas`
- `examples/pascal/record-methods/events.fpas`
- `examples/pascal/record-methods/point.fpas`
- `examples/pascal/record-methods/properties.fpas`
- `examples/pascal/record-methods/rectangle.fpas`
- `tests/debugger/fixtures/capturing_routine_assignment.fpas`
- `tests/debugger/fixtures/function_value_assignment.fpas`
- `tests/runner/static_record_procedure_test.fpas`
- `tests/stdlib/fluent/fluent_records_test.fpas`

Replace `examples/pascal/record-methods/events.fpas` with an explicit optional
handler stored in the button record. Replace the two bound values in
`examples/pascal/record-methods/bound_methods.fpas` with closures over named
receiver snapshots. The callable-field regression keeps direct field calls and
uses an explicit closure for its former bound method.

Move the six files from `examples/pascal/record-methods/` into the existing
`examples/pascal/functions/` directory as `record_counter.fpas`,
`record_point.fpas`, `record_rectangle.fpas`, `receiver_closures.fpas`,
`optional_handlers.fpas`, and `record_accessors.fpas`. Rename
`examples/pascal/generics/generic_record_methods.fpas` to
`generic_record_functions.fpas`, and `examples/pascal/functions/fluent_calls.fpas`
to `function_composition.fpas`.

Move the five `tests/stdlib/fluent/fluent_*_test.fpas` fixtures into
`tests/stdlib/composition/` with the prefix removed. Rename
`tests/runner/static_record_procedure_test.fpas` to `ordinary_procedure_test.fpas`.
Update corpus references, runner registration, documentation and source links
with these moves. Preserve removed spellings only in explicit negative tests.

## Removed implementation paths

Records now contain stored fields only. Parser recovery rejects obsolete member
routines and accessors with an ordinary-function replacement hint, while the
retired words remain usable as identifiers. Positive callable-field and optional
handler fixtures coexist with negative removed-syntax coverage.

Semantic and compiler member resolution, automatic receivers, accessor metadata,
event operations, bound-method construction and fluent-call fallback are removed.
Stored field and indexed callable calls use the ordinary value-call checker.
The formatter emits field access followed by invocation and keeps `.Field(args)`
together when wrapping a chain. Editor symbols, completions, semantic tokens and
TextMate keywords reflect the same model.

IR, bytecode, unit interfaces, object metadata, linker, program codec and debugger
record layouts contain fields only. Ordinary closure captures replace the runtime
bound receiver. Unit format 13, program format 16 and bytecode 17 invalidate old
artifacts; source manifests remain authoritative.

The debugger consumer check reproduced a pre-existing qualified-call issue:
`Holder.Apply(2)` searched only the executable routine catalog. The correction
resolves visible storage and its fields first, retaining catalog fallback only
for an unknown root. JSONL integration covers direct, parenthesized and indexed
stored callables, missing fields, arity mismatch and incompatible arguments.

Current documentation removes the method, property, event and fluent-call pages.
Grammar, records, callables, generic routines, visibility, keywords, formatting,
debugger and editor guidance describe the implemented replacements. TUI, HTTP and
OpenAI API documentation follows the migrated ordinary functions.

## Final verification

- `cargo fmt --all -- --check` and `cargo build --workspace` pass.
- `cargo test --workspace --no-fail-fast` passes: 3,787 tests across 192 groups.
  Superseded member tests are replaced by ordinary-function, closure and
  removed-syntax regressions; binary-format digests match the new layouts.
- FPAS formatting passes for `lib/`, `apps/`, `examples/` and `tests/`.
- The full FPAS bundle passes: 470 tests, one intentional skip, no failures.
- All 22 application/example projects and 109 program entries pass in their
  source or project contexts. Fourteen entries require their project context,
  including the monorepo test program covered by its test manifest.
- All 54 complete handbook programs pass in their source or project contexts;
  three examples are checked together with the units shown on their pages.
- The real VS Code extension host passes diagnostics, formatting, navigation,
  IntelliSense, semantic tools, project workflows, debugger and lifecycle tests.
- Changed Markdown links, Rust documentation targets and the Git diff check pass.

Stage 4 is complete within the approved task migration boundary. Structured task
ownership and the remaining bare-task removal belong to stage 5.
