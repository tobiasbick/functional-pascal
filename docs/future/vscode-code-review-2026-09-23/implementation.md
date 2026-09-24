# VS Code review implementation

Date: 2026-09-24. This document records the follow-up to the findings in
the review. The original reports remain the baseline evidence.

| Finding | Change | Verification |
| --- | --- | --- |
| F01 | Explicit Select Project or Workspace opens the picker even when a project is remembered; ordinary commands reuse the selection. | Host test with an injected picker switches between two manifests and checks ordinary resolution. |
| F02 | `fpas test --file` selects exact discovered paths; JSON results retain full paths. The Testing view maps those paths to distinct nested test items. | Rust sequential and parallel project test with duplicate basenames, plus Testing API request test. |
| F03 | Testing requests expand included parents and subtract excluded descendants; excluding all tests starts no CLI process. | Real TestRunRequest cases for all, one file, excluded child, included parent with exclusions, and excluded root. |
| F04 | Start, stop, and restart operations on the language client are serialized. | Fake-client lifecycle test covers concurrent starts, pending stop, restart, and retry after failure. |
| F05 | A streaming decoder retains incomplete controls, recovers from unsupported CSI, and emits a lone Escape after a short timer. | Every split position for keyboard, mouse, focus, and paste sequences; timeout and recovery cases. |
| F06 | Buttonless mouse motion is a Move event, while dragged motion and releases retain button identity. | SGR mouse action and modifier cases. |
| F07 | Both terminal transports send ordered DAP batches of at most 256 events. | Boundary cases at 256, 257, and 1025 events, including pre-initialization input and adapter failure. |
| F08 | Path identity folds case only on Windows. | Windows identity cases in the host suite; a native Linux Node run creates case-distinct files and checks both identities. |
| F09 | F5 ownership uses a complete TOML parser and the shared manifest index. | TOML basic/literal/multiline/escaped path cases, malformed independent manifests, and duplicate owners. |
| P01 | Terminal text parsing advances a cursor instead of expanding the remaining string. | Input boundary suite and the benchmark below. |
| P02 | Selection and F5 share a cached manifest index; manifests are read concurrently and file/folder changes invalidate snapshots. | Counting test for coalesced queries and stale-scan invalidation. |
| P03 | Workflow captures, terminal input, socket output, and wire messages have explicit byte/event limits; socket writes honor backpressure, and the external client exits after stdout drains. | Capture overflow, input queue, rejected socket authentication, malformed host messages, and large external-client output. |
| M01 | Debug commands share frame selection, expression prompting, and collection requests. | Existing debugger command host tests plus a captured-frame request test. |
| M02 | The VSIX carries bundled extension and external-client entry points; duplicate package inventory checking is removed. | Exact VSIX entry inventory verifier. |
| N01 | Runtime and development dependencies were refreshed and the lockfile replayed from scratch. | Full and production-only `npm audit`: zero findings each; VSIX build and verification passed. |

The terminal benchmark uses the same script and input sizes before and after
the decoder change. Median times for 16,000 repetitions of `x😀` were 2,829.749
ms before and 5.713 ms after; fragmented paste was 5.373 ms before and 0.784
ms after. These are local measurements of the decoder, not end-to-end terminal
latency. Run `node scripts/bench-terminal.mjs` from `editors/vscode` to repeat
the current measurement.

The normal Extension Host suite passed on VS Code 1.137.0 and the declared
minimum 1.91.0. A separate fresh-profile Restricted Mode host launch passed:
the extension remained inactive. A Linux Node process verified path identity on
two case-distinct files. This Linux check does not cover the full Extension Host.

The extension README now describes local-only workspaces, trust behavior,
exact test selection, and explicit buffer limits. CLI test selection and report
paths are documented under `docs/pascal/program-structure/cli.md` and
`docs/pascal/std/testing/test.md`.

Verification from the repository root: `cargo fmt --all`, `cargo build
--workspace --locked --offline -j 1`, `cargo test --workspace --locked --offline
-j 1 -- --test-threads=1`, and `cargo clippy --workspace --all-targets --locked
--offline -j 1 -- -D warnings` passed. The documented `fpas test --file`
example ran one test and passed. From `editors/vscode`, `npm ci`, `npm test`,
`npm run test:restricted`, `npm run package`, both npm audits, and the 1.91.0
host run passed. The final VSIX verifier checked the exact runtime entry list.
No `.fpas` sources were changed.

The queued `ReadLn` host assertion compares DAP `stdout` events. On the 1.91.0
host, a separate post-termination protocol diagnostic can arrive as a DAP
`stderr` event; it is not program output and does not alter the `ReadLn` result.
