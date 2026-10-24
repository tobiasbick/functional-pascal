# Dogfooding gaps

Intake list for gaps found while agents write their helper scripts in FPAS instead of Python,
JavaScript, or shell logic.
Compiler panics also follow the entry rule in
[compiler-panic-followups.md](compiler-panic-followups.md).

## Entry format

Add one entry per gap under [Open entries](#open-entries):

```markdown
### <Short title>

- **Date:** YYYY-MM-DD
- **Kind:** blocker (fell back to another language) or awkward (done in FPAS)
- **Area:** language, `Std.*` API, bug, diagnostics, or performance
- **Task:** what the helper script had to do
- **Missing or wrong:** what FPAS could not do or did badly
- **Minimal example:** the smallest FPAS source that shows the gap
- **Fallback:** the language or tool used instead, if any
```

Remove an entry once the gap is fixed and covered by tests and current documentation.

## Open entries

### Comparing contract execution requires separate source copies

- **Date:** 2026-10-24
- **Kind:** awkward (done in FPAS)
- **Area:** CLI, compiler contracts, performance
- **Task:** Measure the runtime cost of the seven application-level contract
  blocks in Into the Dungeon's client and server.
- **Missing or wrong:** Contracts run in every build mode. There is no CLI policy
  that omits both condition evaluation and checks from the complete program,
  including library artifacts. `fpas run --no-contracts` is not supported.
- **Minimal example:** A function with `requires Value >= 0;` and
  `ensures (Returned): Returned >= Value;` cannot currently be built or run with
  those checks disabled through a CLI option.
- **Fallback:** No other language. An FPAS helper copied sources and manifests
  into two isolated trees, removed only the game's contract blocks in one tree,
  and built both variants. Alternating timing runs and separate profiler runs
  compared the resulting images. Standard-library contracts remained enabled;
  this does not emulate a program-wide contract-disable option. Into the Dungeon
  records the [CLI policy proposal](https://github.com/tobiasbick/fpas-into-the-dungeon/blob/5cc871c9c7d9afe9f2c0f0b7bd7cc8a74db1295f/docs/fpas-wishes.md#completely-disable-contracts-through-the-cli)
  and the [measurement report with raw timing data](https://github.com/tobiasbick/fpas-into-the-dungeon/blob/5cc871c9c7d9afe9f2c0f0b7bd7cc8a74db1295f/docs/bench/contracts.md).
