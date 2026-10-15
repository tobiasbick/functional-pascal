# Checking documentation examples

The FPAS project [tools/doc-examples](../../../tools/doc-examples) extracts and
checks every `pascal` fence in the handbook and the repository README. It runs
automatically as part of `fpas test tests/suite.fpasprj` and the Cargo workspace
tests. Run commands from the repository root.

```text
fpas run tools/doc-examples/doc-examples.fpasprj
fpas run tools/doc-examples/doc-examples.fpasprj -- --document docs/pascal/language/types/records.md
fpas run tools/doc-examples/doc-examples.fpasprj -- --json
```

Each invocation creates a new directory under `.temp-data/doc-examples/` with
the extracted sources, project manifests, compiler diagnostics and `report.json`.
Failures report the Markdown path, source line, example id and diagnostic.
`--json` prints the report as one JSON document. Reports contain `schema`,
`total`, `failed` and a `results` array; a failing check returns a nonzero exit.

## Fence instructions

The first word remains `pascal` for syntax highlighting. Additional words state
how the example is checked. There is no skip option or automatic search for a
wrapper that happens to compile.

| Instruction | Meaning |
| --- | --- |
| `pascal` | Check a complete program or unit; do not execute it |
| `id=NAME` | Stable name within the document, required when another context references it |
| `fragment context=NAME` | Insert an excerpt into the named context template |
| `run` | Check and execute a program with the test runner, strict mode and a five-second timeout |
| `error=FPnnnn` | Require failure with this diagnostic code; with `run`, require it during execution |
| `project=NAME file=PATH` | Group companion sources from the same document into one generated project |
| `manifest=PATH` | Check the actual documented project manifest instead of generating one |

Without an explicit id, the tool reports `line-N`. Use explicit ids for maintained
examples and keep them stable when moving or inserting fences. Names contain only
ASCII letters, digits, `-` and `_`. Output paths are relative, without `.` or `..`
components. Unknown or repeated instructions, missing contexts and duplicate ids
fail the check. Backtick and tilde fences are supported, including CRLF input.

A short excerpt can be checked and executed with the shared `body` context:

```pascal id=checked-fragment fragment context=body run
const Answer := 6 * 7;
if Answer <> 42 then
  panic('unexpected answer');
end if;
```

An intentional compiler error is a separate fence:

```pascal id=rejected-fragment fragment context=body error=FP3006
const Answer: integer := 'forty-two';
```

Every reported error must have the expected code and point into the extracted
example itself. An error in the surrounding context cannot satisfy it.
Unexpected success, another
error code, an execution failure, a timeout or a skipped test fails validation.
Keep valid and rejected examples separate so each is independently checked.
Only use `run` for deterministic, noninteractive examples with assertions or an
explicit expected runtime error. Network, terminal and host-file examples normally
remain compile-only. Execution has the same host access as any FPAS program.

## Contexts and companion projects

Templates live in [tools/doc-examples/contexts](../../../tools/doc-examples/contexts)
as `NAME.fpas.txt`. Exactly one whole line `// @example` receives the source.
The shared `body` template supplies a program header and `begin`/`end.`. Other
templates supply imports, declarations, fixture values or an enclosing routine.
Their purpose is to make the excerpt's stated prerequisites explicit.

A whole line `// @include docs/pascal/PAGE.md#ID` inserts another Pascal fence
from the current Markdown file on disk. Include declaration-only excerpts in
declaration positions; includes insert raw source and do not recursively apply
the referenced fence's context. Prefer this to copying documented declarations.
Optional `contexts/NAME/*.fpas` files supply companion units.

Give all files of a documented project the same `project=NAME`, distinct
`file=PATH` values, and `manifest=PATH` on its Pascal fences. Supporting TOML
fences use `toml project=NAME file=PATH`; they are copied into the generated
directory and validated through the consuming project. A unit on its own is
checked as a library. A unit cannot use `run`; execution requires a program
named `*_test.fpas` (the default generated name for `run`).

Pure notation, API signature outlines, directory trees and output use `text`,
`toml`, or their appropriate language tag. They are not executable FPAS
examples. Do not relabel broken executable examples as text to hide a failure.

## Selecting inputs

`--document PATH` is repeatable and replaces the default handbook/README selection.
Use it to check examples elsewhere in the repository. `--root PATH` selects the
repository containing the handbook and templates. `--std-lib PATH` overrides
the standard library; `--compiler PATH` overrides the current FPAS executable.
`--output PATH` selects a new output directory and refuses to overwrite one.
Keep generated files under `.temp-data/`.

The regression test uses its runner-owned scratch directory, which is cleaned
up after the test. A direct invocation keeps its generated files for inspection.
Formatting remains the responsibility of `fpas fmt`; this check validates
compilation and explicitly selected runtime behavior.
