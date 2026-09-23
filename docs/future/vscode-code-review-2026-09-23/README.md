# VS Code extension review

Date: 2026-09-23. Reviewed commit: `b3d3b84c3c1952364a545d786548eb617e3b596a`.

Scope: `editors/vscode`, including activation, toolchain discovery, project commands, Testing API integration, debugger commands and terminals, grammar, build scripts, packaging, tests, and npm dependencies. Official online documentation was consulted for API and protocol contracts. Rust CLI and DAP implementations were inspected where the extension depends on their behavior.

This review adds documents only. Extension sources, manifests, dependency versions, and the lockfile remain unchanged. Temporary probes and a review VSIX are under the ignored `.temp-data/vscode-review/` directory.

## Reports

- [Correctness and missing tests](correctness.md)
- [Performance and maintainability](performance-and-maintainability.md)
- [Online documentation and local documentation comparison](documentation.md)
- [npm versions and audit](npm-dependencies.md)
- [Verification and reproduction evidence](verification.md)

## Findings

| ID | Priority | Finding |
| --- | --- | --- |
| F01 | P2 | Project selection cannot switch a remembered project through the command palette |
| F02 | P2 | Nested test results lose file identity; selecting one basename can run several files |
| F03 | P2 | Testing API exclusions are ignored |
| F04 | P2 | Concurrent language-client starts leave an unowned client |
| F05 | P2 | Terminal decoding mishandles split escape sequences and stalls on unknown CSI sequences |
| F06 | P2 | Mouse motion without a pressed button becomes a release event |
| F07 | P2 | Terminal input batches exceed the DAP adapter's 256-event limit |
| F08 | P2 | Unconditional case folding merges distinct paths on case-sensitive filesystems |
| F09 | P2 | Partial TOML parsing breaks valid project main paths |
| P01 | P2 | Terminal text decoding repeatedly expands the remaining string |
| P02 | P3 | Project lookup repeatedly scans the workspace and reads manifests serially |
| P03 | P3 | Process output and terminal queues have no retained-byte bound |
| M01 | P3 | Debug commands duplicate frame selection and prompt/request handling |
| M02 | P3 | Packaging includes redundant compiled debugger modules and verifies them twice |
| N01 | P2 | The locked dependency graph has audit findings, including bundled brace-expansion |

P2 denotes incorrect behavior or a concrete reliability problem worth fixing. P3 denotes maintenance or scaling work. Source-derived performance findings are not measured speedup claims. F01-F09 have focused JavaScript or CLI reproduction evidence, with platform and mocking limitations described in the reports.

## Verification

TypeScript compilation, manifest checks, contract checks, grammar checks, the real VS Code 1.137.0 Extension Host suite, VSIX creation, and package verification passed. The passing tests omit the reproduced selection, nested-test, concurrency, and terminal edge cases.

Six of eleven direct npm dependencies have newer stable releases. The full npm audit reports five affected package entries: four high and one moderate. The production-only audit reports one high entry. These are dependency findings, not five demonstrated exploits in this extension.

Recommended first work: correct test identity and selection, serialize language-client lifecycle operations, repair terminal decoding/batching, and refresh the vulnerable dependency graph. Preserve the existing host tests and add the missing boundary cases before refactoring shared code.
