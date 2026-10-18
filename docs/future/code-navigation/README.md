# N1–N3: source queries for the agent

Temporary delivery plan for the [navigation wishes](../agent-wishes.md#code-navigation).
The user accepted this direction and delegated all open tooling decisions to the
agent that will use it. The [contract](01-contract.md) records those decisions;
no implementation or new command has shipped yet. This is not an AP package.

## Goal

I need to identify a declaration, find its actual uses and inspect checked types
without maintaining an LSP connection or guessing from text search. The normal
workflow is one source position in, one self-contained JSON result out.

Deliver references first, then definitions, then expression types. Reuse the
language service that already serves the editor. Source ownership, resolution and
checked types belong there; the CLI owns request parsing, lifecycle and JSON.
This does not change the FPAS language.

## Decisions

- Canonical commands: `fpas query refs`, `fpas query definition`, `fpas query type`.
- One required `path:line:column`; positions match FPAS diagnostics: one-based
  lines and Unicode-scalar columns. No alternate byte/UTF-16 position switch.
- JSON is the only query result format, including request failures. No report flag.
- Resolve by declaration identity, with explicit manifest scope and coverage.
  An empty reference list must never hide missing analysis or an unbound target.
- N1 lists uses only; the declaration is always a separate `target` field.
- N3 returns a real checked type, never a type guessed from hover text.
- JSONL removal remains deferred. DAP, JSONL and LSP retain their contracts.
- N4 rename, record/replay and scheduling changes are outside this delivery.

## Progress and implementation order

This table owns the only completion checklist. A decided contract is not an
implemented feature, and delivery does not imply a commit or push.

| Done | Step | Status |
| --- | --- | --- |
| [x] | [01: shared command/result contract](01-contract.md) | Decided under the user's delegation |
| [ ] | [02: N1 references and shared CLI foundation](02-references.md) | Next; not started |
| [ ] | [03: N2 definition](03-definition.md) | After 02 |
| [ ] | [04: N3 checked types](04-type.md) | After 02 and 03 |

There are no unresolved product questions. Implementation findings are resolved
within this contract; do not silently weaken completeness or substitute textual
heuristics when existing analysis needs a focused extension.

## Delivery rules

Each step includes meaningful owner-level and real CLI tests, help and permanent
documentation, independent review, fixes and an updated status before the next
step. Reuse existing tests where they already own the behavior. Never rebuild a
CLI executable while process tests are using it.

For Rust changes run formatting, build and workspace checks, including the
existing language-service/LSP and D3 regression owners when affected. Report
pre-existing exclusions, ignored helpers and unrun platforms accurately. Cloud
timings are not performance evidence.

The permanent contract belongs in `docs/pascal/program-structure/cli.md` and a
focused tooling page once implemented. Check the compact reference and authoring
skill too. Mark each wish implemented only when its step passes acceptance.
Remove this directory and its future-index links after all three steps are
shipped and the handbook owns their complete contract. Commit/push only when
separately requested.
