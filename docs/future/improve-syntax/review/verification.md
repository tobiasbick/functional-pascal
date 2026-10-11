# Review repair verification

The [repair index](README.md) tracks six findings from reviewed revision
`34397373799a5897dfcfc09e754d16fcad3c7386`. Closed items require a demonstrated
regression, an owner-boundary repair, a second code review and applicable checks.
Current handbook rules remain unchanged.

## Required checks

These results cover the currently completed repairs S01, S02, S03, T01 and T02.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | Passed |
| `cargo build` | Passed |
| `cargo test --workspace --no-fail-fast` | 4,187 tests/doc-tests passed; no failures or ignored tests |
| `cargo clippy -p fpas-sema --all-targets --locked -- -D warnings` | Passed |
| `cargo clippy -p fpas-language-service --all-targets --locked -- -D warnings` | Passed |
| `node editors/vscode/scripts/run-tests.mjs` | Full extension-host suite passed, including manifest, contract and grammar checks |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test tests/suite.fpasprj --std-lib lib` | 500 passed, 1 intentional skip, no failures |
| Local Markdown links and heading fragments | Passed |
| `git diff --check` | Passed |

CLI checks use the current debug build, copied to `.temp-data/` while running
the standalone suite to avoid locking Cargo's executable on Windows. Rust and
standalone FPAS suites run sequentially to avoid concurrent fixture discovery.
The editor host runs with `ELECTRON_RUN_AS_NODE` unset; an initial environment
failure launched the host as Node and was resolved without repository changes.

## Regression and second-review evidence

| Finding | Evidence and status |
| --- | --- |
| S01 | Both new semantic regressions failed before repair because invalid programs had no errors. All 13 focused decision-expression tests now pass. The real CLI table checks FP3006, expected/found types and semantic phase for check/build/run/test. The runner confirms valid Result values and selected-branch-only evaluation. Second review confirms shared recursive merging, branch-context validation and unchanged no-widening rules. |
| S02 | All three new semantic regressions and the runtime fixture failed before repair with the documented contextual callback mismatch. All 37 focused generics tests now pass. Record and enum callbacks execute without annotations. Second review confirms shared contextual traversal, retained missing/conflicting-evidence rejection and generic constraints; enclosing generic parameters remain valid evidence. |
| S03 | New semantic negatives failed before repair because type names were accepted; the CLI regression reported FP9001 instead of FP3006. Direct and computed arguments now produce semantic diagnostics. All three conversion-argument tests, five membership tests and the CLI regression pass. Second review includes type names hidden in decisions, addition and negation, plus local/imported aliases and preserved membership operands. |
| T01 | Public API regressions failed before repair by selecting an outer type and, on second review, selecting an owner name as the parameter span. All three targeted tests pass: record/enum definition, references, rename and tokens; constrained and same-owner parameters; exclusion from qualified member lookup/completion. Edited positive sources remain valid. Shared extraction and member filtering avoid query-specific fixes. |
| T02 | Four public identity regressions failed before repair by resolving binders to the outer constant. All five targeted tests pass for case/if expressions, sibling and nested arms, guards, `and` chains, statement patterns and return/initializer/argument/container/closure positions. Second review checks earlier selector and pattern-comparison uses, declaration visibility and independently valid renames of outer and inner bindings. Shared pattern extraction owns both statement and expression routes. |
| T03 | Reviewed public signature help reports template types instead of instantiated types. Repair open. |

The remaining editor probes ran against the reviewed revision through the
public `LanguageService` API after semantic analysis accepted their inputs.
Their source inputs and acceptance contracts remain in
[tooling-and-tests.md](tooling-and-tests.md). Permanent regressions must
demonstrate each failure before its production repair.

## Scope and limits

- No keyword, grammar, highlighting or current handbook rule changed in S01/S02/S03/T01/T02.
  Existing language-service and LSP tests pass in the workspace suite.
- The full VS Code extension-host suite passes for T01 and T02, including navigation,
  IntelliSense, semantic tools, project workflows, debugger and lifecycle checks.
- No benchmark or exhaustive feature-combination claim is made.
- An incidental T02 fixture exposed a separate navigation limitation: a field
  in `Box(...).Value` can resolve to an outer `Value`. This postfix lookup is
  outside the six repairs; the binder fixture uses a differently named field
  to isolate the requested identity checks.
- Link validation uses an FPAS helper under `.temp-data/`; reports contain no
  machine-identifying metadata.
