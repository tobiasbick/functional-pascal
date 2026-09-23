# Tests and verification

## Completed checks

| Command | Result |
| --- | --- |
| `target/debug/fpas.exe fmt --check apps examples lib` | Passed |
| `target/debug/fpas.exe test --jobs 4 --report json tests/suite.fpasprj` | 436 passed, 1 skipped, 0 failed; 437 total |
| `cargo test -p fpas-cli --locked --offline example_` | 61 passed, 0 failed, 412 filtered in the matching test target |
| `target/debug/fpas.exe check apps/notes/notes.fpasprj` | Passed |
| `target/debug/fpas.exe check apps/local-chat/local-chat.fpasprj` | Passed |
| `target/debug/fpas.exe check lib/stdlib.fpasprj` | Rejected by normal project loading because `Std` is reserved |
| `target/debug/fpas.exe check .temp-data/fpas-review/doc-example/my-app.fpasprj` | Expected failure confirming D01, F2003 for `Add` |

The standard library is normally loaded as part of the toolchain through `lib/stdlib.fpasprj`. Directly checking that manifest as an ordinary project is not a substitute for this loading path. The suite and application checks exercise standard-library consumers; the rejected command is recorded as a verification limitation, not as evidence that the library fails to compile normally.

Only review documents were added. A full Rust workspace build/test cycle was not repeated for this documentation-only change. The Rust example tests above were run because they exercise checked-in FPAS examples. Interactive examples were not all launched; the repository's curated example test selection avoids blocking terminal and network demos.

## Reproductions performed

Ignored probes and fixtures were created under `.temp-data/fpas-review/`. They are local investigation artifacts and are not required to read this report. No probe was added to the permanent test suite.

### Notes and URI public API probe

A small program depended on `apps/notes/notes-core.fpasprj` and called the public repository/update APIs plus `Std.Net.Uri.Parse`. Its output was:

```text
ftp://127.0.0.1 -> rejected
ftp://127.0.0.1:21 -> accepted ftp:21
http://127.0.0.1:8_0 -> accepted http:80
http://127.0.0.1:+80 -> accepted http:80
initial save failed=true
retry dirty=false failure overlay=true
save-and-quit after retry=false
loaded=3 issues=0
select second -> first
bracket directory loaded=1
loaded path=.temp-data/fpas-review/notes-fixture/set1/outside.note
```

The three loaded notes consist of the successfully retried draft plus two files with duplicate IDs. The bracketed-directory case uses separate empty `set[1]` and populated `set1` directories. See F01-F03 and F06 for setup and expected outcomes.

### HTTP loopback probe

A local socket fixture returned controlled HTTP responses and recorded request lines. The client used public `Std.Http.Send` through a request to `/base/start`. Each response case completed successfully, which exposed the incorrect accepted behavior:

| Fixture | Observed behavior |
| --- | --- |
| Redirect to `/target/` | Next request uses `/target` |
| Redirect to `/a//b` | Next request uses `/a/b` |
| Redirect to `..` | Next request uses `/`, expected control case |
| `Content-Length: +2` with `ok` | Body accepted |
| `Content-Length: 1_0` with ten bytes | Body accepted |

All fixture cases used loopback and terminated successfully. No internet service or model endpoint was involved.

## Missing regression coverage

| Area | Existing coverage inspected | Needed case |
| --- | --- | --- |
| Notes repository | `tests/apps/notes/note_repository_roundtrip_test.fpas` | Literal directory with glob metacharacters; assert no sibling load or save |
| Notes identity | `note_selection_after_save_test.fpas`, format tests | Two valid files sharing an ID; deterministic issue or correct distinct selection |
| Notes recovery | `note_navigation_dirty_test.fpas`, `note_tui_workflow_test.fpas` | Failed save followed by repaired destination, successful retry, navigation and quit |
| URI parsing | `tests/stdlib/net/uri_parse_test.fpas` | Unsupported scheme with and without explicit port; signs and separators in ports |
| HTTP redirects | Rust CLI network hardening tests | Assert exact redirected request target for trailing and repeated slashes |
| HTTP framing | HTTP limit and hardening tests | Reject language-specific numeric syntax in both request and response Content-Length |
| SSE limit | `tests/stdlib/net/sse_final_limit_test.fpas`, streaming tests | State and retained input after oversized feed; repeated calls after error |
| UTF-8 | `tests/stdlib/net/utf8_roundtrip_test.fpas` | Broader invalid sequence boundaries; separately benchmark doubling input sizes |
| Documentation | Runnable project examples | Compile the Markdown project examples themselves |

Existing coverage is useful: the complete FPAS suite passed, and the example integration selection passed. These results establish a baseline but do not negate the independently reproduced failures.

## Suggested order

1. Add the F01 confinement regression and fix literal directory loading.
2. Cover and fix Notes identity and retry transitions.
3. Add loopback framing/redirect regressions and URI validation cases.
4. Define the SSE state after errors and test retained buffers.
5. Correct the documentation examples and stale shutdown paragraph.
6. Measure the identified performance paths before changing their representation.

Any implementation follow-up should use the normal project change checklist.
