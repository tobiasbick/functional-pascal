# Checked FPAS refactorings

`fpas refactor rename` prepares a checked JSON edit plan in an explicit `.fpasproj`
or `.slnx` context. `fpas refactor apply` validates and applies that plan. Both
use compiler symbols and the project's actual references.

```sh
fpas refactor rename Example.fpasproj --file main.fpas --line 5 --column 10 --new-name Calculate --plan .temp-data/rename.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/rename.json --report json
```

## Selection and names

Select a declaration or one of its checked references with --file/--line/--column.
Coordinates are one-based UTF-16 as in [source queries](../program-structure/cli.md).
Use --project to select an owning FPAS compilation for a linked source file.
The search still covers every analyzed FPAS project in the context. All owners
of an edited shared declaration participate in validation.

--new-name takes one semantic ASCII identifier, without qualification or @,
with at most 1,024 characters. The compiler chooses canonical keyword escaping:
a rename to end writes @end. A single underscore is a wildcard, not a new name.
Names are case-sensitive. Renaming to the existing name produces an empty plan.
For program/unit names, selection renames the last component; files and project
names are not renamed.

Supported source declarations include containers, records, interfaces, unions,
domains, aliases/import aliases, routines, fields/properties, globals, union cases,
enum members, parameters/type parameters, and local/pattern bindings. Selecting
a routine renames its resolved overload and callable references. Connected source
interface members and their implementations form a rename family. A family that
requires changing an external CLR contract is refused.

Namespace-only and implicit/generated bindings and declarations outside the
editable FPAS context cannot be selected. Comments, string values and C#/F#
sources remain outside the edit set. Public API changes are identified in the
plan; unknown consumers and foreign-language bodies are not checked or rewritten.
Documentation validation can require manual comment changes after a parameter
rename.

## Validation and plans

The original context must analyze successfully. Rename finds exact declaration
and reference tokens, retains their original text and deduplicates linked edits.
Proposed source is checked with the same compiler. FPAS dependency images are
compiled into an owned temporary directory for this validation, without source
writes or application execution. Temporary images are removed afterwards.

Compilation success alone is insufficient: every indexed source occurrence must
retain its declaration/metadata binding after offsets are adjusted. Source
interface relationships must also remain the same. Name collisions, changed
overload selection and accidental capture of unrelated symbols refuse the plan.
The validation is limited to the compiler's indexed source facts and declared
context, rather than reflection strings or unknown consumers.

The plan contains version/kind, context/configuration, input snapshot, original
selection, semantic new name, affected declarations, exact edits, original/result
file hashes, source projects, binary boundaries and a canonical SHA-256 identity.
The .json output is created without overwriting an existing file. Inspect the
plan or the CLI's readable edit list before applying.

Apply requires the original context/configuration. It checks plan identity and
input snapshot, regenerates the entire rename from compiler facts, and requires
the regenerated plan to match. Supplied edit lists are never applied directly.
Source/context bytes and the loaded plan file are reverified before publication.
Changes to source, projects, references, build inputs or compiler can invalidate
a plan; prepare a new one for the changed context.

## Publication and reports

Application shares [source publication](diagnostics.md#apply-automatic-corrections)
with automatic corrections: staged replacements, byte checks, preserved UTF-8 BOM,
line endings, Unix permission bits and Windows access rules, and rollback/recovery
reporting. Symbolic links/reparse points and read-only edited sources are refused.
Files not edited retain their timestamps. Replacement is atomic per file; the
batch does not lock other applications.

After application, the context is reloaded, dependencies prepared and sources
checked again. Reports distinguish written files from successful rechecking.
If cancellation/setup failure follows publication, status remains applied with
success false, rechecked false and explicit written flags. Recovery failures list
remaining written paths and retained files. Preparation can restore/build ordinary
dependencies; it follows the [check contract](../program-structure/cli.md).

--configuration defaults to Debug; --no-restore skips restore only. --timeout
covers the whole operation, defaults to 120 seconds and accepts 1..3600. Cancellation
stops and joins owned build processes. Preview/apply work is bounded to 1,000 edit
occurrences, 100,000 indexed occurrences, 64 MiB of distinct source input,
64 MiB of changed original/replacement bytes and 64 MiB of temporary dependency
images. Plans and JSON reports are bounded to 4 MiB.

Human output shows the plan path and edits; diagnostics use stderr. --report json
owns stdout with one object, including failures. It contains success/status,
planPath/plan, per-file written flags, rechecked, diagnostics, error and recovery
paths. Exit codes: 0 success/help, 1 conflict/stale plan/setup/publication/deadline
failure, 2 invalid arguments/ambiguous source owner, 130 cancellation.

## VS Code refactorings

The VS Code commands prepare edits against the checked live workspace, including
unsaved buffers and each owner of a linked file. The preview lists exact edits
and their original ranges. Apply recomputes the edit against the current input
snapshot, then submits one versioned text edit for the open buffers. The editor
owns undo and saving. An affected closed file leaves the result preview-only.
Reprepare after opening it, changing an input, switching context or restarting
the language server.

- **Add Parameter** accepts a name, an explicit FPAS type and one value
  expression for checked direct calls to a private unit, program or record routine.
  It inserts the parameter before existing defaults and labels positional
  arguments that cross the insertion. Each new value argument is evaluated
  after the existing explicit arguments. The routine body stays intact.
  Routines with attributes, `params`, external contracts, callable references
  or untracked callers require a manual signature change.
- **Extract Variable** creates a typed local `const` in the same block.
  Select an expression evaluated first in an initializer, return, raise,
  initial condition or case selector. Extraction can traverse a leading
  operand or first argument of a checked unqualified call. A later or guarded
  operand, loop condition, indirect receiver, managed argument, lambda or
  await requires manual editing.
- **Organize Imports** removes unused CLR type and namespace aliases when all
  source owners agree. It preserves source-unit imports, comments, line
  endings and the order of surviving imports. Entries adjacent to comments
  stay in place when deletion would consume the comment.

All three check the proposed overlay with the actual project references.
Existing source bindings and relationships must survive the edit. A changed
overload, invalid type, name collision or changed input prevents application.
Plans expire after five minutes and each application consumes its preview.

## See also

- [Compiler and source analysis CLI](../program-structure/cli.md)
- [Diagnostics and automatic corrections](diagnostics.md)
- [Workspace architecture](../../architecture/workspaces.md)
