# STE terminology and avoid-list

Companion reference for [fpas-docs-ste](../SKILL.md). Use one canonical term
for each concept across all Markdown files.

## Words and phrases to avoid

Replace vague, wordy, or ambiguous wording with direct alternatives.

| Avoid | Use instead | Reason |
| --- | --- | --- |
| `simply`, `just`, `easily`, `basically`, `actually`, `obviously` | Omit | Filler words add noise and dismiss difficulty. |
| `please note that`, `note that`, `keep in mind that`, `it is important to` | State the rule directly | Front-load the actual rule. |
| `utilize`, `leverage`, `facilitate` | `use`, `enable`, or a specific verb | Prefer plain verbs. |
| `perform execution of`, `make a call to`, `do a check of` | `run`, `call`, `check` | Avoid nominalizations. |
| `in order to` | `to` | Shorter and clearer. |
| `prior to` / `subsequent to` | `before` / `after` | Plain prepositions. |
| `in the event that` / `provided that` | `if` | Direct condition. |
| `due to the fact that` / `as a result of` | `because` | Direct cause. |
| `via` | `with` or `through` | Clearer English preposition. |
| `a number of`, `various`, `several different` | Exact count or `several` | Be specific. |
| `is able to`, `has the ability to` | `can` | Direct modal verb. |
| `allows you to`, `enables you to` | `lets you` or imperative verb | Shorter and active. |
| `make sure` | `verify` or `ensure` | Clear technical verb. |
| `should` (in contracts or rules) | `must` or simple present tense | Avoid ambiguous obligation. |
| `currently`, `at this time`, `for now`, `previously`, `now` | Omit | Document current implemented behavior only. |
| `etc.`, `and so on` | Name the concrete items or state the category | Avoid open-ended ambiguity. |
| `thing`, `stuff`, `handler` (when vague) | Exact type, callback, or component name | Name the actual construct. |

## FPAS language and CLR terms

See [FPAS for agents](../../../../docs/pascal/agents/README.md) for the
language summary.

| Canonical term | Avoid | Usage |
| --- | --- | --- |
| `program` | `script`, `entry file` | Executable source declared with `program Name; ... end program;`. |
| `unit` | `module`, `namespace file` | Library source declared with `unit Name; ... end unit;`. |
| `routine` | `subroutine`, `callable` | General term covering `function` and `procedure`. |
| `entry routine` | `top-level API`, `public boundary` | Program main routine or public library operation where `requires`/`ensures` belong. |
| `binding` (`const`, `var`) | `local`, `let`, `immutable variable` | Use `const` for non-rebindable bindings and `var` for rebindable bindings. |
| `record` | `struct`, `class`, `data object` | FPAS reference record type declared with `record ... end record;`. |
| `variant` / `variant case` | `sum type`, `enum branch`, `tag` | Discriminated union declared with `variant ... end variant;` and its qualified cases. |
| `domain` | `newtype`, `strong typedef` | Distinct nominal wrapper type. |
| `contract` (`requires`, `ensures`) | `assertion`, `guard clause` | Precondition (`requires`) and successful-return postcondition (`ensures`). |
| `uses` alias | `import alias`, `using directive` | Explicit alias introduced by `uses Namespace as Alias;`. |
| `discard` | `ignore statement`, `void cast` | Explicit `discard Call();` statement that drops a function result. |
| `nullable` | `optional`, `maybe` | CLR nullable value or reference type (`T?`). |
| `assembly` | `binary`, `package` (when referring to compiled output) | Emitted CLR `.dll` output. |
| `documentation sidecar` | `doc XML`, `xmldoc file` | Exported `.NET` XML documentation file beside the assembly. |

## Effects, workflows, and concurrency

| Canonical term | Avoid | Usage |
| --- | --- | --- |
| `Effect of (A, E, R)` | `async thunk`, `IO monad` | Deferred work in `Fpas.Execution.Effects` with success `A`, expected failure `E`, and environment `R`. |
| `expected failure` | `typed error`, `soft error` | Failure returned as `Outcome.Failure`. |
| `unexpected CLR exception` | `crash`, `untyped error` | Thrown .NET exception that faults the task. |
| `workflow` | `policy runner`, `orchestrator` | Execution policy or structured lifetime in `Fpas.Execution.Workflows`. |
| `cancellation token` | `cancel handle`, `abort signal` | Explicit .NET `CancellationToken` forwarded to async APIs. |
| `scoped resource` | `auto-disposed object` | Resource bound with `using const` or `await using const`. |

## MVU terms

See [Fpas.Ui.Mvu](../../../../docs/ui/mvu/runtime.md) for the runtime contract.

| Canonical term | Avoid | Usage |
| --- | --- | --- |
| `model` | `app state`, `store` | Immutable application state passed to `View` and `Update`. |
| `view` | `render tree`, `template` | Pure function that builds `Element of TMsg` from the model. |
| `message` | `event`, `intent`, `signal` | Typed `TMsg` value dispatched to `Update`. |
| `update function` / `Update` | `reducer`, `event handler` | Pure function returning `Transition of (TModel, TMsg)`. |
| `transition` | `update result`, `state change` | `Mvu.Transition of (TModel, TMsg)` containing the next model and commands. |
| `command` | `side effect`, `action` (for `Command of TMsg`) | `Command of TMsg` returned by `Update` for the host to run. |
| `host request` | `host command`, `native call` | Request produced by `Focus`, `Exit`, or `HostRequest.Extension`. |
| `mailbox` | `event queue`, `channel` | Bounded FIFO queue fed by `Send` and `Post`. |
| `owner` / `owner turn` | `UI thread` (in host-neutral MVU docs), `tick` | Single execution context calling `Advance`, `Dispatch`, `Snapshot`, and `Close`. |
| `background effect` | `worker task`, `async job` | Deferred work started by `StartBackground` and cancelled by `CancelBackground`. |
| `slice` | `sub-app`, `feature module` | Independently updated MVU feature with its own model, messages, update, and view. |
| `subview` / `cached subview` | `child component`, `memoized view` | Composed child element tree mapped with `Fpas.Ui.Composition.Map` or `CachedSubview`. |
| `stale reply` | `outdated message`, `late response` | Queued reply from a superseded or cancelled background effect. |
| `EffectReplies` | `mock effects`, `reply stub` | `Fpas.Ui.Mvu.Testing.EffectReplies` helper for deterministic reply ordering in tests. |

## GUI, TUI, and CUI terms

See [Fpas.Ui](../../../../docs/ui/widgets.md),
[Fpas.Ui.Gui](../../../../docs/ui/gui/host.md), and
[Fpas.Ui.Tui](../../../../docs/ui/tui/host.md).

| Canonical term | Avoid | Usage |
| --- | --- | --- |
| `GUI host` / `TUI host` | `backend`, `runner`, `engine` | Concrete desktop (`Fpas.Ui.Gui`) or terminal (`Fpas.Ui.Tui`) host loop. |
| `adapter` | `wrapper`, `bridge` | Host-specific integration layer connecting `Fpas.Ui` and `Fpas.Ui.Mvu` to CUI or a terminal. |
| `element` | `node`, `VDOM node` | Declarative `Element of TMsg` value produced by a view. |
| `widget` | `component` (in FPAS UI docs) | Reusable declarative UI building block under `Fpas.Ui.Widgets`. |
| `control` | `interactive widget`, `input element` | Focusable or interactive widget with a stable ID. |
| `stable ID` / `widget ID` | `key` (except sidebar/menu keys), `handle` | Nonempty, unique, case-sensitive control, modal, or cached-subview ID kept across frames. |
| `capability profile` | `target platform`, `feature tier` | `Portable`, `Terminal`, or `Graphical` validation profile in `Fpas.Ui.Validation.Prepare`. |
| `logical units` | `pixels` (for layout values) | Host-independent GUI sizing units before DPI scaling. |
| `cells` | `characters` (for TUI geometry) | Terminal grid units, with `80 × 25` as the baseline SSH viewport. |
| `software renderer` | `CPU backend`, `fallback renderer` | Deterministic CUI software renderer used for captures and off-screen checks. |
| `headless session` | `mock host`, `fake terminal` | Non-interactive session driven with `Send`, `Step`, `Snapshot`, and `CloseAsync`. |
| `theme` / `system appearance` | `skin`, `OS dark mode` | `Fpas.Ui` theme palette/metrics and CUI-reported light or dark appearance. |
| `overlay` | `popup layer`, `floating window` | Modal or anchored surface such as a dialog, sheet, or popover. |
| `focus` | `active element`, `selection` (when referring to keyboard focus) | Keyboard target managed by the host and requested with `Focus`. |

## Toolchain and documentation terms

| Canonical term | Avoid | Usage |
| --- | --- | --- |
| `CLI` (`fpas`) | `command-line tool`, `compiler tool` | Public `fpas` command-line interface. |
| `SDK` (`Fpas.NET.Sdk`) | `MSBuild plugin`, `build extension` | Local MSBuild SDK that selects `.fpas` sources and invokes the compiler. |
| `.fpasproj` project | `FPAS manifest` (except in `doc-examples` `manifest=PATH`) | MSBuild project file for an FPAS executable, library, or test assembly. |
| `.slnx` solution | `workspace file`, `solution file` (when format matters) | XML solution grouping FPAS and C# projects. |
| `complete program` / `fragment` | `full snippet` / `partial snippet` | Complete compilable `program`/`unit` fence versus `fragment context=NAME` fence. |
| `companion unit` | `helper file`, `supporting snippet` | Unit compiled alongside a program in a documented `project=NAME` group. |
| `context template` | `wrapper file`, `harness template` | `NAME.fpas.txt` template with `// @example` used by `doc-examples`. |

## Rewrite examples

**Verbose:**
> Please note that in order to simply make sure that background work does not
> cause outdated events to be applied to your app state, you should utilize
> stable effect IDs so that starting a new job will automatically cause the
> cancellation of the previous one.

**STE:**
> Use stable effect IDs. Starting another background effect with the same ID
> supersedes the previous run, invalidates its queued messages, and requests
> cancellation.

**Verbose:**
> This function allows you to easily map child component events into parent
> actions via `MapTransition`.

**STE:**
> Use `Fpas.Ui.Mvu.Composition.MapTransition` to translate a child transition
> into parent messages and commands.

**Verbose:**
> Note that TUI layouts should currently be designed so that they are able to
> fit nicely inside a standard terminal window of 80 characters by 25 lines.

**STE:**
> Design terminal views for an 80 × 25 cell viewport and keyboard input.
