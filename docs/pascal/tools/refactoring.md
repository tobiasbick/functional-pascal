# Checked FPAS refactorings

`fpas refactor` prepares checked JSON edit plans for rename, parameter addition,
removal and signature changes, protocol implementation, argument labels,
variable extraction and inlining, routine extraction and import organization.
Supply an explicit `.fpasproj` or `.slnx` context.
`fpas refactor apply` validates and applies the selected plan.

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

Supported source declarations include containers, records, interfaces, variants,
domains, aliases/import aliases, routines, fields/properties, globals, variant cases,
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

## Public signatures across projects

Use a project or solution context containing the declaring FPAS project and
every consumer. A project context follows its dependencies, so use a solution
when separate projects depend on the same declaration. For a public
`add-parameter`, `remove-parameter` or `change-signature` plan, supply
`--closed-context` to confirm that the selected context contains all consumers.
The confirmation is stored in the plan.

FPAS resolves source declarations behind references and verifies each affected
FPAS project against freshly compiled in-memory metadata. It refuses affected
C# or F# project consumers, captured binary boundaries that reference an affected
assembly, and interface contracts. The selected context bounds what FPAS can
verify. Account for consumers outside it, including reflection or dynamically
loaded code, before confirming the change.

```sh
fpas refactor change-signature Suite.slnx --file Library/Api.fpas --line 5 --column 17 \
  --mode ref --argument 'Client/Main.fpas:18:11=Current' --closed-context \
  --plan .temp-data/signature.json --report json
fpas refactor apply Suite.slnx --plan .temp-data/signature.json
```

## Add a parameter

Select a unit, program or record routine by its name or a checked call
to it. Pass the unescaped name and explicit FPAS type for the new parameter
and a value expression that is valid at every direct call.

```sh
fpas refactor add-parameter Example.fpasproj --file main.fpas --line 5 --column 10 \
  --name Limit --type int --argument '10' --plan .temp-data/add.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/add.json
```

The declaration gets a required value parameter before the existing defaults.
The plan labels existing positional arguments where needed and adds the new
argument after all existing explicit arguments. The routine body stays intact.
Review the plan's `addedParameter` inputs and edits before applying it. Apply
regenerates the edits from the stored request and checks every analyzed owner.
Routines with attributes, `params`, an external contract, callable references,
or untracked callers require a manual signature change.

## Remove an unused parameter

Place the cursor on the parameter declaration or a named argument that resolves
to it. Preview the edits, then apply the saved plan.

```sh
fpas refactor remove-parameter Example.fpasproj --file main.fpas --line 5 --column 20 --plan .temp-data/remove.json
fpas refactor apply Example.fpasproj --plan .temp-data/remove.json
```

The operation removes one unused value parameter from a unit, program or
record routine. It removes the matching positional or named argument at each
direct call. Calls that use the parameter's default can keep their original text.
The remaining arguments retain their written order, parameter types and overload
binding in every analyzed project that owns the source.

The compiler must prove that skipping argument evaluation and conversion preserves
behavior. Simple literals and ordinary local or value-parameter reads qualify.
For example, removing the second parameter from `Choose(Keep, 0)` can produce
`Choose(Keep)`. An argument such as `ReadNext()` or `Total div Count` needs a
manual decision because evaluating it may have effects or throw. A used
parameter (including a contract or capture), a routine used as a delegate,
an external contract, a variadic or attributed routine, or a deletion spanning
a comment also prevents automatic removal.

## Change a parameter type or mode

Select a parameter of a unit, program or record routine by its declaration,
reference or named argument. Supply an explicit `--type`, `--mode`, or both.
Modes are `value`, `in`, `ref` and `out`.

```sh
fpas refactor change-signature Example.fpasproj --file main.fpas --line 5 --column 17 \
  --mode ref --argument '18:11=Current' --plan .temp-data/signature.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/signature.json
```

FPAS updates the declaration and writes the requested mode at each direct call.
Existing expressions and labels are retained. When a call needs a new expression,
repeat `--argument [FILE:]LINE:COLUMN=EXPRESSION` for the affected calls.
Coordinates identify the called name. With no file, they refer to the routine's
source file. Use an absolute caller path or one relative to the working
directory to address another file. Supply only the expression.
FPAS writes the label and mode where needed. An explicit replacement for an
omitted argument is appended as a named argument after the written arguments.

The compiler verifies the unchanged routine body, defaults and every direct
call against the proposed signature. The plan preserves existing overload
selection, generic substitutions and the order of untouched expressions.
Managed modes require addressable storage, and `out` requires definite
assignment. If an existing expression or a default cannot satisfy the new
signature, use a valid explicit call replacement or edit the incompatible
declaration manually. Attributed routines, `params`, delegates, external
contracts require a manual signature change.

## Reorder parameters

Select a routine name or a direct call and use `--order` with every unescaped
parameter name in the requested order. Keep required parameters before optional
ones. Use a separate plan for type or mode changes and argument replacements.

```sh
fpas refactor change-signature Example.fpasproj --file main.fpas --line 5 --column 10 \
  --order 'Right,Left,Weight' --plan .temp-data/parameter-order.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/parameter-order.json
```

For a routine originally declared with `Left, Right, Weight`, a call such as
`Combine(Next(), Read(), Weight := 2)` becomes
`Combine(Left := Next(), Right := Read(), Weight := 2)`. Arguments are evaluated
once in their original written order. Default values and passing modes move
with their declarations. Existing names and omitted arguments stay as written.
The plan records the full order in `signatureChange.order`.

The same [closed-context rules](#public-signatures-across-projects) apply to
public routines. The compiler rejects changed call targets or generic
substitutions. Attributed and variadic routines, delegate uses and external
contracts require manual changes. A comment next to reordered parameters
prevents automatic reordering when its attachment would be ambiguous.

## Implement a protocol

Select a record name or a reference to it. Pass a FPAS protocol or CLR
interface type as written in the record's unit, including type arguments.
The compiler resolves inherited requirements and reuses compatible public
instance methods, getters and existing default interface implementations.

```sh
fpas refactor implement-protocol Example.fpasproj --file Store.fpas --line 18 --column 13 \
  --protocol 'IStore<int>' --forward 'Read=ReadValue' --member 'Count=return 1;' \
  --member "Label='Local store'" --plan .temp-data/store.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/store.json
```

A request that omits required behavior lists the missing members, their
signatures and their keys. Supply each key once. Overloaded missing methods
use keys such as `Read#1`. Use the keys shown in the diagnostic. Supply one or
more FPAS statements for a method, including their semicolons. The command
supplies the enclosing routine block. Use one FPAS initializer expression for
a getter-only property. A property becomes a public record field with that
initializer. Use `--forward MEMBER=METHOD` to call an existing instance method
on the selected record with the contract's arguments.

The command adds imports needed by generated signatures. It checks the
implementation, all affected FPAS projects and existing bindings before
writing a plan. The plan stores the type, supplied bodies and forwarding
requests in `protocol` for regeneration on apply. A protocol the record already implements produces an empty
plan. Changes to record construction or existing overload selection can
prevent automatic application.

## Name arguments at a call

Use `fpas refactor name-arguments` to preview labels for the positional arguments
of one call. Place the cursor on the callee name. Inside a nested call, the
innermost call takes precedence.

```sh
fpas refactor name-arguments Example.fpasproj --file main.fpas --line 8 --column 14 --plan .temp-data/names.json
fpas refactor apply Example.fpasproj --plan .temp-data/names.json
```

For example, `Text.Substring(Offset, Count)` becomes
`Text.Substring(startIndex := Offset, length := Count)`. At least two supplied
parameters must share an input type. Generic substitution, nullable references
and array inputs for spans follow the [API usage template rules](api-documentation.md#generate-checked-fpas-usage).

The preview inserts only the missing labels. Expressions, comments, existing
labels, argument modes and written evaluation order stay intact. A named
`params` argument can supply an array. Expanded variadic elements have no
individual label. Parameter names must be usable and distinct, and every owner
of a linked file must agree on the insertions. The compiler rechecks the target,
overload and concrete call signature before the edit can be applied.

## Extract a local constant

Use `extract-variable` with a complete expression range and the unescaped name
of the new local constant. Provide inclusive start and exclusive end positions
with one-based lines and UTF-16 columns.

```sh
fpas refactor extract-variable Example.fpasproj --file main.fpas --line 8 --column 10 \
  --end-line 8 --end-column 24 --name Items --plan .temp-data/extract.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/extract.json
```

FPAS inserts a typed local `const` in the same block and replaces the selected
expression with its name. Select an expression evaluated first in an
initializer, return, raise, initial condition or case selector. Extraction can
traverse a leading operand or the first argument of a checked unqualified call.
A later or guarded operand, loop condition, indirect receiver, managed argument,
lambda or await requires manual editing.

## Inline a local variable

Select a local `const` or `var` by its declaration or one of its reads.

```sh
fpas refactor inline-variable Example.fpasproj --file main.fpas --line 8 --column 11 \
  --plan .temp-data/inline.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/inline.json
```

The plan replaces the checked reads with the grouped initializer and removes
the original declaration. It can replace several reads of a literal. An
initializer that needs evaluation must have a single read that is evaluated
first in the immediately following statement of the same block. For example,
`const Answer: int := Read();` followed by `return Answer + 1;` becomes
`return (Read()) + 1;`.

The initializer and each read must have the local's declared type. The compiler
checks that bindings, dispatch and generic substitutions survive the edit.
A write, storage use, implicit conversion at the declaration, or a comment
that would be lost or duplicated prevents automatic inlining. The command is
for ordinary local bindings. Resource declarations and initializers containing
`await` or a routine remain explicit.

## Extract a routine

Select complete, consecutive statements from one block, including the final
semicolon and any attached comments. Give the unescaped name of the nested
helper and an exclusive end position.

```sh
fpas refactor extract-routine Example.fpasproj --file main.fpas --line 8 --column 3 \
  --end-line 10 --end-column 19 --name UpdateTotal --plan .temp-data/routine.json --report json
fpas refactor apply Example.fpasproj --plan .temp-data/routine.json
```

The helper runs at the original location. It uses FPAS lexical captures to
read outer values and write the same outer variables in the original order.
If one local declared in the selection is used afterward, select through its
declaration as the final statement. The helper returns its initializer to the
original caller binding. A selection ending in a synchronous return can move
the return statements into the helper and return from the caller at the call
site.

Select the entire loop for its `break` or `continue`, the entire handler for
a bare `raise`, and the owning block for a `using` declaration. The body being
moved must stay synchronous and leave nested or anonymous routine declarations
in their original scopes. The compiler checks captures, initialization, types,
control flow and call targets in every owner. A use that depends on caller-only
flow refinement or an outer managed reference parameter can prevent extraction.

## Organize imports in a file

Use `organize-imports` with a source file. For a linked file, select an analysis
owner with `--project` when needed.

```sh
fpas refactor organize-imports Example.fpasproj --file main.fpas --plan .temp-data/imports.json
fpas refactor apply Example.fpasproj --plan .temp-data/imports.json
```

FPAS removes only unused CLR type and namespace aliases when all owners agree.
It preserves source-unit imports, comments, line endings and the order of
surviving entries. Plan notes describe imports retained to preserve adjacent
comments. A file with no safely removable imports produces an empty plan.

## Plan and apply a checked draft

Use the same source manifest accepted by
[check and query](../program-structure/cli.md#check-a-draft-in-memory).
Drafts can fix an invalid starting point. All source changes are analyzed together
and FPAS dependencies are built as private in-memory metadata for that analysis.
Draft planning always uses this preparation mode with already available external
references and assets.

```sh
fpas refactor draft App.slnx --overlay draft.json --plan .temp-data/draft-plan.json --report json
fpas refactor apply App.slnx --plan .temp-data/draft-plan.json --report json
```

When preparing from an earlier draft check, pass its ID as `--expected-snapshot`.
The plan stores the original input snapshot, the checked `draftSnapshotId` and
the exact replacements for changed owned `.fpas` files. Review the stored edits
before applying. The shared 4 MiB plan limit includes the old and new texts.
New sources record their expected owner and full text in `createInProject` and
`createText`.

Apply regenerates the validation from the plan and current inputs, rechecks the
original bytes and publishes through the standard edit transaction. The manifest
is needed only for planning. A changed original or a different context rejects
the plan. When adding a source, application checks that the path is still free
and re-evaluates ownership. It rolls back the batch if the actual compilation
inputs or checked result differ from the planned draft. Draft validation covers
the selected context and its FPAS dependencies.

## Validation and plans

For symbol refactorings, the original context must analyze successfully. Rename finds exact declaration
and reference tokens, retains their original text and deduplicates linked edits.
Proposed source is checked with the same compiler. FPAS dependencies are compiled
to private in-memory metadata images for this validation, without writing their
DLLs/PDBs, changing sources or executing the application.

Compilation success alone is insufficient: every indexed source occurrence must
retain its declaration/metadata binding after offsets are adjusted. Source
interface relationships must also remain the same. Name collisions, changed
overload selection and accidental capture of unrelated symbols refuse the plan.
The validation is limited to the compiler's indexed source facts and declared
context, rather than reflection strings or unknown consumers.

The plan contains version/kind, context/configuration, input snapshot, original
selection, operation inputs such as the added parameter or extraction range
and name, affected declarations, exact edits, original/result file hashes,
source projects, binary boundaries and a canonical SHA-256 identity.
Use `--expected-snapshot` with the ID from a checked query when preparing a
plan from its source coordinates. The precondition is checked before interpreting
the coordinates or creating the plan. See
[require a snapshot](../program-structure/cli.md#require-a-snapshot).

For rename, add or remove parameter, change signature, implement protocol and
inline variable, a [`query find` match](../program-structure/cli.md#reuse-a-found-symbol)
can select the declaration. Supply `--symbol`, `--project` and
`--expected-snapshot` in place of source coordinates. The plan retains the
resolved declaration position and uses the ordinary validation when applied.

The .json output is created without overwriting an existing file. Inspect the
plan or the CLI's readable edit list before applying.

Apply requires the original context/configuration. It checks plan identity and
input snapshot, regenerates the requested refactoring from compiler facts, and requires
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

After application, the context is reloaded and sources are checked again. Drafts
use the same in-memory dependency analysis for that check. Reports distinguish
written files from successful rechecking.
If cancellation/setup failure follows publication, status remains applied with
success false, rechecked false and explicit written flags. Recovery failures list
remaining written paths and retained files. Preparation can restore/build ordinary
dependencies; it follows the [check contract](../program-structure/cli.md).

`--configuration` defaults to Debug. `--no-restore` skips restore only.
`--existing-inputs` also skips dependency builds and preparation hooks. `--timeout`
covers the whole operation, defaults to 120 seconds and accepts 1..3600. Cancellation
stops and joins owned build processes. Preview/apply work is bounded to 1,000 edit
occurrences, 100,000 indexed occurrences, 64 MiB of changed original/replacement
bytes and a 64 MiB retained cache of overlay analysis and dependency images.
Rename also limits distinct source input to 64 MiB. Plans and JSON reports are bounded to 4 MiB.

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

- **Add Parameter** previews the same [parameter addition](#add-a-parameter)
  with a name, FPAS type and call argument.
- **Remove Parameter** previews the [unused parameter removal](#remove-an-unused-parameter)
  for the declaration or named argument under the cursor.
- **Name Arguments** previews the [parameter labels](#name-arguments-at-a-call) for the call under the cursor.
- **Extract Variable** previews the same [typed extraction](#extract-a-local-constant)
  for the selected expression.
- **Organize Imports** previews the same [file import organization](#organize-imports-in-a-file).

Each refactoring checks the proposed overlay with the actual project references.
Existing source bindings and relationships must survive the edit. A changed
overload, invalid type, name collision or changed input prevents application.
Plans expire after five minutes and each application consumes its preview.

## See also

- [Compiler and source analysis CLI](../program-structure/cli.md)
- [Diagnostics and automatic corrections](diagnostics.md)
- [Workspace architecture](../../architecture/workspaces.md)

## Source files

<details>
<summary>Browse source files</summary>

- `src/Fpas.Cli/Refactoring/`
  - [RefactorCommand.cs](../../../src/Fpas.Cli/Refactoring/RefactorCommand.cs)
  - [RefactorReport.cs](../../../src/Fpas.Cli/Refactoring/RefactorReport.cs)
- `src/Fpas.Compiler/Analysis/`
  - [NamedCallArguments.cs](../../../src/Fpas.Compiler/Analysis/NamedCallArguments.cs)
  - [ProtocolImplementation.cs](../../../src/Fpas.Compiler/Analysis/ProtocolImplementation.cs)
  - [RefactoringSyntax.Routine.cs](../../../src/Fpas.Compiler/Analysis/RefactoringSyntax.Routine.cs)
  - [RefactoringSyntax.Inline.cs](../../../src/Fpas.Compiler/Analysis/RefactoringSyntax.Inline.cs)
  - [RefactoringSyntax.cs](../../../src/Fpas.Compiler/Analysis/RefactoringSyntax.cs)
- `src/Fpas.Workspaces/Refactoring/`
  - [CheckedRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/CheckedRefactoring.cs)
  - [ExtractionRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/ExtractionRefactoring.cs)
  - [FileRefactorPlans.cs](../../../src/Fpas.Workspaces/Refactoring/FileRefactorPlans.cs)
  - [DraftCreation.cs](../../../src/Fpas.Workspaces/Editing/DraftCreation.cs)
  - [ImportRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/ImportRefactoring.cs)
  - [InlineRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/InlineRefactoring.cs)
  - [MovedOperations.cs](../../../src/Fpas.Workspaces/Refactoring/MovedOperations.cs)
  - [NameArgumentsRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/NameArgumentsRefactoring.cs)
  - [ParameterOrder.cs](../../../src/Fpas.Workspaces/Refactoring/ParameterOrder.cs)
  - [ParameterRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/ParameterRefactoring.cs)
  - [ParameterContext.cs](../../../src/Fpas.Workspaces/Refactoring/ParameterContext.cs)
  - [ProtocolRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/ProtocolRefactoring.cs)
  - [RefactorPlan.cs](../../../src/Fpas.Workspaces/Refactoring/RefactorPlan.cs)
  - [RenameBindings.cs](../../../src/Fpas.Workspaces/Refactoring/RenameBindings.cs)
  - [RenameOverlay.cs](../../../src/Fpas.Workspaces/Refactoring/RenameOverlay.cs)
  - [RenameService.cs](../../../src/Fpas.Workspaces/Refactoring/RenameService.cs)
  - [RoutineExtraction.cs](../../../src/Fpas.Workspaces/Refactoring/RoutineExtraction.cs)
  - [SignatureRefactoring.cs](../../../src/Fpas.Workspaces/Refactoring/SignatureRefactoring.cs)
- `tests/Fpas.IntegrationTests/Refactoring/`
  - [AddParameterTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/AddParameterTests.cs)
  - [ChangeSignatureTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/ChangeSignatureTests.cs)
  - [CliEditingRefactoringsTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/CliEditingRefactoringsTests.cs)
  - [ExtractRoutineTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/ExtractRoutineTests.cs)
  - [InlineVariableTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/InlineVariableTests.cs)
  - [ImplementProtocolTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/ImplementProtocolTests.cs)
  - [NameArgumentsTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/NameArgumentsTests.cs)
  - [ParameterOrderTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/ParameterOrderTests.cs)
  - [PublicParameterTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/PublicParameterTests.cs)
  - [RemoveParameterTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RemoveParameterTests.cs)
  - [RenameCliTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RenameCliTests.cs)
  - [RenameFixture.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RenameFixture.cs)
  - [RenameGraphTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RenameGraphTests.cs)
  - [RenamePlanTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RenamePlanTests.cs)
  - [RenameSemanticTests.cs](../../../tests/Fpas.IntegrationTests/Refactoring/RenameSemanticTests.cs)

</details>
