---
name: fpas-authoring
description: Write, fix, or format Functional Pascal .fpas sources, including programs, units, examples, and regression tests.
---

# FPAS authoring

Start with the [compact language reference](../../../docs/pascal/agent-reference.md)
for syntax, common Pascal/Delphi/Ada pitfalls, and complete tested examples. Use the
[topic reference](../../../docs/pascal/README.md) and nearby working sources for
details. Repository placement and shared completion checks live in
[AGENTS.md](../../../AGENTS.md).

## Read for the task

| Task | Reference |
|------|-----------|
| Programs, units, imports, visibility | [Units](../../../docs/pascal/program-structure/units.md) |
| Bindings and expressions | [Basics](../../../docs/pascal/language/basics/README.md) |
| Routines, parameters, closures | [Functions](../../../docs/pascal/language/functions/README.md) |
| Branches and loops | [Control flow](../../../docs/pascal/language/control-flow/README.md) |
| Standard-library calls | [Std reference](../../../docs/pascal/std/README.md) |
| Assertions, scripted input, golden output | [Test runner](../../../docs/pascal/std/testing/test.md) |

Read the pages relevant to the change. For exact productions, consult
[grammar.ebnf](../../../docs/specs/grammar.ebnf). For the exact signature and
documentation of a `Std` routine, a built-in operation or a project declaration, run
`fpas doc <name>` (for example `fpas doc Std.Fs.ReadText` or `fpas doc string.TrimLeft`);
see [Looking up declarations](../../../docs/pascal/program-structure/cli.md#looking-up-declarations).

## Source workflow

1. Inspect the owning manifest and neighboring sources before choosing a program
   or unit. Use [fpas-projects](../fpas-projects/SKILL.md) when manifests or
   dependencies need editing.
2. Reuse a working pattern: [hello world](../../../docs/pascal/getting-started/hello-world.md),
   [assertion test](../../../tests/stdlib/str/higher_order_test.fpas), or
   [headless TUI test](../../../tests/stdlib/tui/mvu_host_signature_test.fpas).
3. For a new regression test, confirm that
   [tests/suite.fpasprj](../../../tests/suite.fpasprj) includes its path.

## Syntax to keep explicit

- Each type, constant, or variable repeats its own `type`, `const`, or `var` keyword. Repeat `public` on each exported declaration.
  Use `const` for immutable values and `var` for reassignment.
  Parameters and loop variables are read-only; copy a parameter into a fresh
  local `var` when it needs local changes or shared mutable captures.
  Computed `const` bindings are allowed inline in statement lists;
  their initializer runs whenever execution reaches the declaration. Keep record
  fields and enum members in their existing syntax; local types are not allowed.
- Statement-local bindings may infer any fully determined initializer type,
  including calls, operators, field/index reads, and if/case expressions. Empty
  collections, None, and incomplete Result constructors need context. Inference
  never promotes integer to real. Declarations before begin, parameters, results,
  and record fields require annotations.
- Separate declared parameters with semicolons and call arguments with commas.
  Calls are fully positional or fully named; fixed-signature callable values use
  their visible static parameter labels. Arguments evaluate in written order,
  even when names reorder parameters. Variadic calls are positional.
- Scalar `case` value labels and both range endpoints require compile-time
  constants. Calls and computed bindings belong in guard conditions.
- Patterns bind explicitly: `when Some(const Value):`,
  `when Shape.Rect(const W, _):`, and `when const N if N > 0:` for a scalar
  guard binding. `_` ignores one field. Patterns nest (`Ok(Some(const User))`)
  and compare payloads with literals or compile-time constants; a plain
  identifier that names no constant is an error. Each payload list is fully
  positional or fully named, covering every field once; Some/Ok/Error use Value
  as their payload label. Coverage is checked
  recursively. Closed enum, Option, and Result cases list every variant
  explicitly and reject `else`, including a redundant branch. Only scalar
  integer/string cases require `else`; Boolean and subrange cases cover their
  complete finite domain and forbid `else`. Statements and expressions share
  these rules; distinct follows its supported base type. Guards never count
  toward coverage. Duplicate labels within an arm and labels fully covered by
  earlier unguarded arms are rejected; partial overlaps remain legal.
  `Value is Some(_)` is a normal
  Boolean expression supporting grouping, logical operators, guards, loops,
  arguments and returns. It never binds names; `const Name` is rejected at any
  depth. Use case arms to extract payloads. `is` is reserved. Return binders
  require the colon in `ensures (Returned): Expression;`; `ensures (Flag);`
  is an ordinary grouped Boolean condition.
- Import every referenced unit with `uses Unit.Path as Alias;`, including `Std.*`.
  Access imported public names only through the source-local alias.
  Import each unit only once per file, comparing names case-insensitively.
  `uses Std.Math as Math;` exposes only `Math.Name`, with no short names or
  access through `Std.Math.Name`. Alias names cannot be shadowed by declarations,
  parameters, loop variables, or pattern bindings. `as` remains contextual in `uses`.
- Qualify every user enum variant with its visible type: `Color.Red` locally or
  `Domain.Color.Red` through an import. Use the same form in construction,
  comparisons, nested patterns, and `is` tests. Some/None/Ok/Error keep their
  reserved language forms; explicit local type aliases and native operations
  retain their existing rules.
- Functions return with `return`. Consume function results or use an allowed
  explicit [discard](../../../docs/pascal/language/functions/discard.md).
- Import `uses Std.Parse as Parse;` and call Parse.Int/Real/Bool/HexInt,
  each taking Text: string and returning Result of (target type, Parse.ParseError).
  Std.Conv formats and converts typed
  values. Use try for propagation, nonbinding is for variant tests, and exhaustive
  case for payload binding, local handling, or plain defaults. Unwrap deliberately
  aborts on Error/None; try does not catch runtime failures. Map transforms wrapped
  success, AndThen composes wrapped operations, and OrElse recovers within a wrapper.
  Callback argument expressions are eager; callback bodies run only on their
  selected variant. Case fallback work runs only in its selected arm.
  Try requires the same wrapper family in the enclosing function; success types
  may differ. Result propagation preserves compatible errors, nominal identity,
  and the same generic error parameter. Program bodies and procedures cannot use try.
- Go is one expression and requires a lexical scope in the same routine.
  Bind its exact task handle or use `discard go Work();`; named and anonymous
  helpers never inherit ownership. Pass TaskGroup for dynamic caller-owned work.
  Discard accepts every value type without task/capture proofs and never cancels,
  detaches, joins, or consumes a task result. Foreign mutable variables, including
  globals through helpers/callbacks, are forbidden in workers; pass values,
  mutate worker-local copies, or use channels. Dynamic targets retain VM checks.
  Creating tasks cancel and fully close remaining groups before publishing their
  outcome. Returned Error is Result data; retrieve required results before close.
  Wait consumes once across aliases; completion observers do not consume.
  Controlled waits validate arguments/original failures first, then observe
  cancellation, readiness, timeout on every probe. Select uses ready-case input order.
- Named free, member, and nested routine signatures are available throughout
  their lexical scope, including mutually recursive siblings. Values initialize
  in source order. Call or capture a helper only after its direct and transitive
  dependencies are initialized; FP3041 rejects unsafe or unprovable use.
  Callable bindings remain ordinary initialized values. Record defaults run
  at construction in their declaring environment.
- Native collection mutation requires InPlace: Items.PushInPlace(Value),
  Items.SortInPlace(), or Values.RemoveInPlace(Key). Without InPlace, operations
  preserve receivers: Push returns a new array; Pop returns ArrayPopResult of T
  with Remaining and Value. PopInPlace returns the removed element. Both Pop forms
  fail on empty arrays. InPlace needs writable storage and retains its static type;
  MapInPlace/FlatMapInPlace cannot change element/value types. User Self stays read-only.
- Read string scalars with `Text[Index]`; use `in` for collection membership.
  Repetition uses RepeatStr/RepeatStrInPlace with arbitrary strings; an API that
  requires one scalar states that precondition explicitly. Array IndexOf/FindIndex
  and string IndexOf/LastIndexOf return Option of integer, so handle Some/None.
  SortBy/SortByInPlace take F: function(X: T): K with Comparable keys; record
  elements are allowed. They sort ascending and stably, evaluating F once per
  snapshot element in input order (zero times when empty).
  Sort/SortInPlace are ascending and stable and require Comparable elements,
  including payload-free enums; typed empty unsupported arrays fail checking.
  Their generic elements need T: Comparable or T: Numeric.
- Statements end with `;`. Use each construct's named closer, such as
  `end function;`, `end if;`, and `end unit;`; programs end with `end.`.
  Expression closers receive their terminator from the enclosing statement.
- Unit declarations and record members are private by default; `public` exports
  an individual declaration or member in a unit.
- Construct records through their type with named fields, for example
  `Point(X := 1, Y := 2)`. Omit fields only when they have defaults; use
  `TypeName()` for empty records or to use all defaults. Supplied values run in
  written order, followed by omitted defaults in declaration order.
- Strings use single quotes and doubled quotes for escaping. Source comments use
  `//`; adjacent standalone blocks provide [Markdown documentation](../../../docs/pascal/language/basics/comments.md).

## Verify sources

Use [formatter output](../../../docs/pascal/tools/fmt-style.md) as the style authority:

Use two-space structural indentation. Keep meaningful argument and expression
groups with authored line breaks, then run the formatter. It uses a fixed
100-column expression target, automatically expands long calls and collections,
and keeps postfix suffixes attached. Multiline statement conditions put `then`
on a separate line; wrapped `and`, `or`, and `xor` use separate continuation lines.
CLI and editor formatting share this style without configuration.

```text
fpas fmt <paths>
fpas fmt --check <paths>
fpas check <file-or-project>
fpas check --fix <file-or-project>
fpas test <test-path>
```

`fpas check --fix` applies the compiler's machine-applicable fixes for Pascal,
Delphi and Ada habits in place; format afterwards and fix what remains by hand.

Check interactive programs without launching them in batch validation. Use
scripted or headless tests for their runtime behavior. After FPAS regression-test
changes, run the repository suite as required by AGENTS.md.

CLI text diagnostics include a bounded source excerpt and caret when matching
text is available. Use `--diagnostics json` for structured records. Redirected
text stays plain; nonempty `NO_COLOR` disables automatic terminal colors.

For an escaping runtime error, run with
`fpas run <file-or-project> --failure-dump .temp-data/<fresh-name>.json`.
The [bounded report](../../../docs/pascal/tools/failure-dump.md) keeps the original
error and available task/frame/value data. Inspect `capture.incomplete` and
omission reasons before assuming a value was absent. Success creates no report;
the destination must be a new file.

For handbook examples, follow the [Markdown example instructions](../../../docs/pascal/tools/documentation-examples.md).
Complete `pascal` fences compile by default; fragments name an explicit context.
Mark deterministic assertion programs `run` and separate intentional errors with
their exact `error=FPnnnn` code. The normal suite checks the actual Markdown.

## Inspecting source bindings

Use `fpas query definition file.fpas:12:5` to identify a declaration,
`fpas query refs file.fpas:12:5` to find its uses, and `fpas query type
file.fpas:12:5` to inspect a checked value type. Select a project or workspace
with `--context <manifest>`. Check `coverage.complete` before treating empty
matches as absence; exit 3 carries incomplete analysis or a bound. Read the
[source-query contract](../../../docs/pascal/tools/source-queries.md) for selection
rules, snapshot provenance and limits.

## Inspecting a failure or stop

Use `fpas run file.fpas --failure-dump .temp-data/new-failure.json` to retain
an escaping failure from the normal concurrent scheduler. For a targeted all-stop
inspection, use `fpas debug file.fpas --break file.fpas:42 --print Total`.
The default command returns one bounded JSON document with stacks, values and
captured output; it never reads host stdin. Repeat `--print` for independent
non-executing expressions and use `--script input.toml` for runner-compatible
line/key input. See [batch debugging](../../../docs/pascal/tools/debugger-batch.md)
for limits and exit codes. DAP/JSONL remain available through explicit protocol flags.
For a compiled target, pass `--source-root` with the `Debug source root` printed
by `fpas build`; it covers the program and resolved source dependencies, including
sibling projects. Rebuild older images with upward-relative source paths.

To explain how a scalar value changed, repeat `--trace Total`,
`--trace State.Selected` or exact `--trace 'Routine::Name'` on the batch command. The
[tracepoint contract](../../../docs/pascal/tools/debugger-tracepoints.md) covers
typed initialization/change events, actual writing locations, exact scalar
comparison and incomplete history. Mutable globals/fixed scalar record fields
and scalar locals/parameters/captures or their stored record fields are supported, for
example `--trace 'Update::State.Selected'`, or fixed array positions such as
`--trace 'Update::Items[0]'`. Absent positions are known unavailable values, not
missing history. Local activation/instance identities
distinguish calls and repeated declarations; observation ends at scope/call exit.
Concrete source projections retain applied generic field types and nominal
array element names through aliases, references and captures. Unresolved generic
parameters remain unsupported; never infer types from values/display strings.
Use exact `MakeCounter.Next::Total` to select a capture during each active
callback call. Each admission starts a new instance; dormant changes are outside
that history. Anonymous routines use deterministic source-ordered names such as
`MakeCounter.$closure_1` per lexical owner; root literals use `$entry_programname`.
Copy the full name from the stack and single-quote shell selectors; identical
compilation inputs retain names, but inserting earlier literals can renumber them.
Whole aggregate selectors are unsupported. `--trace-limit`
defaults to 1,000 events and stops explicitly at a bound; inspect `trace.complete`
before relying on an empty or partial history. Tracing executes no expressions.

For task/channel/timer ordering, use `fpas debug file.fpas --schedule-seed 42`.
The [seeded scheduling contract](../../../docs/pascal/tools/debugger-scheduling.md)
uses fair task rounds, virtual task time and controlled effects. Supply fixed
arguments and text-line scripts; key scripts and uncontrolled host effects are
rejected. Pure synchronous callbacks work, but their task/sleep/scope operations
reject. Check `schedule.status` and reasons, and retain the executable, runtime
build/platform, inputs, limits and seed when repeating. This is not replay of the
normal concurrent worker pool; DAP/JSONL keep their existing behavior.
