# Verification and reproduction evidence

## Completed checks

Commands below run from `editors/vscode` unless noted otherwise.

| Check | Result |
| --- | --- |
| `npm ci --ignore-scripts --no-audit --no-fund` | Installed the locked dependencies; no manifest/lockfile change. |
| `npm run compile` | Passed TypeScript checking/emission and esbuild bundling. |
| `node scripts/verify-manifest.mjs` | Passed. |
| `node scripts/verify-contracts.mjs` | Passed. |
| `node scripts/verify-grammar.mjs` | Passed. |
| `npm test` | Passed in the real VS Code 1.137.0 Extension Host, exit code 0. |
| `node node_modules/@vscode/vsce/vsce package --no-dependencies --out ../../.temp-data/vscode-review/review.vsix` | Created a 24-entry VSIX, about 193 KB compressed. |
| `node scripts/verify-package.mjs ../../.temp-data/vscode-review/review.vsix` | Passed archive inventory, metadata, and path checks. |
| `npm outdated --json` and per-package `npm view` | Six direct stable updates found. Exit 1 from outdated indicates outdated packages. |
| `npm audit --json` | Five affected package entries. |
| `npm audit --omit=dev --json` | One high runtime entry, brace-expansion. |

The first restricted-network attempts to contact npm and download VS Code failed with `EACCES`. The checks were rerun successfully with network access. This was an environment restriction, not an extension failure. The host runner builds `fpas-cli` itself, and that build passed.

The Extension Host suite emits one aggregate success message covering diagnostics, formatting, navigation, IntelliSense, semantic tools, workflows, debugger, and lifecycle. It does not expose a trustworthy numeric test-case count, so none is invented here.

The review used the writing guidance from the `unslop` skill. No implementation skill or source rewrite was needed.

## Focused probes

`node .temp-data/vscode-review/probe.cjs`, from the repository root, loads compiled source modules with small VS Code and LanguageClient test doubles. It produced these results:

| Probe | Observed result |
| --- | --- |
| Reselect with two projects and first remembered | First returned; picker called 0 times. |
| Exclude one of two discovered tests | Both still requested. |
| Select one nested test | CLI filter contains only `same_test.fpas`. |
| Map a nested basename-only JSON result | 0 recorded statuses. |
| Two concurrent LSP starts, then stop | 2 clients created; 1 stopped. |
| Split ESC from arrow suffix | Escape plus literal `[` and `A`. |
| Unknown CSI followed by ordinary text | No events; input remains blocked. |
| Mouse code 35 | `Up` with no button instead of `Move`. |
| Send 257 ordinary characters through integrated terminal manager | One 257-event request. |
| TOML triple-quoted literal main path | Extra quote characters remain in the resolved path. |
| Remember lowercase path beside uppercase variant | First case-insensitive match selected. |
| Existing terminal decoder tests | Passed. |

The tests with mocks establish implementation behavior at the tested boundary. They do not establish real Linux filesystem behavior, native-terminal rendering, or the number of native LSP processes during a user interaction.

A separate real CLI project under `.temp-data/vscode-review/nested/` contains `a/same_test.fpas` and `b/same_test.fpas`. Discovery returns two absolute paths. `fpas test --report json --filter same_test.fpas suite.fpasprj` runs both and reports two indistinguishable basename entries. Both test programs pass. This confirms the CLI contract assumption used in the nested-result probe.

## Missing permanent tests

- Project switching through the command palette without an explicit URI.
- Nested test directories, duplicate basenames, exact selected-file execution, and every result mapped back to a discovered item.
- Testing API include/exclude combinations through a real run profile.
- Concurrent LSP startup, restart during activation, stop while startup is pending, and toolchain changes during startup.
- Terminal sequences split at every byte boundary, unknown-sequence recovery, movement without buttons, and bounded multi-request input batching.
- Case-distinct paths on a case-sensitive filesystem.
- Valid TOML string spellings in debugger ownership discovery.
- Minimum declared editor version and Restricted Mode activation behavior.
- External terminal authentication/error paths and large-output completion, beyond the integrated host tests.

No permanent tests or implementation files were added. Probes and the review archive are ignored investigation artifacts. The existing Rust and FPAS review directories were left unchanged. This review does not claim full cross-platform coverage, measured performance improvements, or verified compatibility of the available dependency updates.
