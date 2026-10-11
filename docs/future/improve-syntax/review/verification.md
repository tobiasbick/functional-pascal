# Review repair verification

The [repair index](README.md) tracks six findings from reviewed revision
`34397373799a5897dfcfc09e754d16fcad3c7386`. Closed items require a demonstrated
regression, an owner-boundary repair, a second code review and applicable checks.
Current handbook rules remain unchanged.

## Required checks

These results cover the currently completed repair S01.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | Passed |
| `cargo build` | Passed |
| `cargo test --workspace --no-fail-fast` | 4,172 tests/doc-tests passed; no failures or ignored tests |
| `fpas fmt --check lib apps examples tests` | Passed |
| `fpas test tests/suite.fpasprj --std-lib lib` | 500 passed, 1 intentional skip, no failures |
| Local Markdown links and heading fragments | Passed |
| `git diff --check` | Passed |

CLI checks use the current debug build, copied to `.temp-data/` while running
the standalone suite to avoid locking Cargo's executable on Windows. Rust and
standalone FPAS suites run sequentially to avoid concurrent fixture discovery.

## Regression and second-review evidence

| Finding | Evidence and status |
| --- | --- |
| S01 | Both new semantic regressions failed before repair because invalid programs had no errors. All 13 focused decision-expression tests now pass. The real CLI table checks FP3006, expected/found types and semantic phase for check/build/run/test. The runner confirms valid Result values and selected-branch-only evaluation. Second review confirms shared recursive merging, branch-context validation and unchanged no-widening rules. |
| S02 | Reviewed probes reject both named-field orders and enum payloads with FP3006. Annotated/direct-field controls run successfully. Repair open. |
| S03 | Reviewed subrange/distinct type-name arguments reach FP9001. Repair open. |
| T01 | Reviewed public definition query finds no generic record parameter. Repair open. |
| T02 | Reviewed public definition query selects the outer constant instead of the arm binder. Repair open. |
| T03 | Reviewed public signature help reports template types instead of instantiated types. Repair open. |

The remaining editor probes ran against the reviewed revision through the
public `LanguageService` API after semantic analysis accepted their inputs.
Their source inputs and acceptance contracts remain in
[tooling-and-tests.md](tooling-and-tests.md). Permanent regressions must
demonstrate each failure before its production repair.

## Scope and limits

- No keyword, grammar, highlighting or current handbook rule changed in S01.
  Existing language-service and LSP tests pass in the workspace suite.
- The full VS Code extension-host suite and Clippy have not been rerun for
  these repairs. Affected editor checks belong to the tooling findings.
- No benchmark or exhaustive feature-combination claim is made.
- Link validation uses an FPAS helper under `.temp-data/`; reports contain no
  machine-identifying metadata.
