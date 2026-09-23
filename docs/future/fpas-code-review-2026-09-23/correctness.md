# Correctness findings

Paths and line numbers refer to the reviewed commit. Suggested fixes below are proposals only.

## F01, P1: Notes interprets directory names as glob syntax

Source: `apps/notes/src/Notes/Repository.fpas:65`, especially line 68; saving at lines 24 and 110.

`NotesLoadDirectory` builds a glob by joining the literal directory with `*.note`. Glob metacharacters in the directory remain active. The repository accepts every returned file and retains its full path in `Note.Path`. Subsequent saves use that retained path.

Reproduction through the Notes repository API:

1. Create empty sibling directories `set[1]` and `set1` under a scratch root.
2. Save a valid note only in `set1`.
3. Call `NotesLoadDirectory` with `set[1]`.
4. The returned collection contains the note from `set1`, with that sibling path in `Note.Path`.

Expected: the selected literal directory is empty. Actual: another directory supplies the editable note. This contradicts `apps/notes/README.md:22`, which promises confinement to the selected directory. The load was reproduced; the outside-directory save consequence follows from the existing path-preserving save implementation.

Use literal directory enumeration followed by extension filtering, or a correctly escaped glob prefix. Preserve intentional glob behavior in the general `Std.Fs.Glob` API. Add cases for bracketed directories, spaces, and adjacent directories containing valid notes. Verify both loaded paths and the destination of a subsequent save. This is an application manifestation of the filesystem glob concern also discussed in the Rust review.

## F02, P2: Successful save retry leaves Notes in the failure state

Source: `apps/notes/src/Notes/Update.fpas:72`, `:115`, `:136`, and `:264`.

The error branch sets `Overlay := SaveFailure` and `ErrorMessage`. The success branch updates the note, clears `Dirty`, and changes `Status`, but keeps the failure overlay and old error. Navigation, creation, and Save-and-quit inspect that overlay and refuse to continue even after persistence succeeds.

Reproduction: make the target directory a regular file, save a dirty draft, then repair the target and retry. The retry returns `Dirty = false` and `Overlay = SaveFailure`. Sending action 17, Save-and-quit, emits no quit command. The probe repaired the target by changing the model directory to an existing writable scratch directory.

Clear the save failure overlay and its error after a successful retry. Preserve unrelated overlays as required by the UI flow. Test failure, repair, successful retry, then navigation and Save-and-quit. Existing successful-save tests do not establish recovery behavior.

## F03, P2: Duplicate persisted IDs make selection resolve to the first note

Source: `apps/notes/src/Notes/Repository.fpas:65`; `apps/notes/src/Notes/Update.fpas:24`, `:37`, and `:115`.

Directory loading accepts two valid files with the same metadata ID. Selection then turns the selected row into an ID and calls `NoteIndexById`, which returns the first match. Copying a `.note` file can produce this condition without malformed file contents.

Reproduction: save `first.note` and `second.note`, both with ID `duplicate` and different titles. Directory loading reports no issues. Build the model with these two notes and select row 1 through `TuiMsgSelectionChanged`. The selected title is `first`.

Reject duplicate IDs as visible repository issues, or use an unambiguous file identity throughout selection and mutation. Decide which duplicate remains editable without silently dropping either file. Add a two-file regression covering load, selection, save, and archive. Existing selection-after-save coverage uses distinct identities.

## F04, P2: Redirect normalization removes meaningful path separators

Source: `lib/Std/Http/Redirect.fpas:70`, especially the empty-segment branch at line 84 and its callers at lines 142 and 158.

`NormalizePath` drops every empty segment before rebuilding the path. This removes trailing slashes and collapses repeated slashes. Servers may route the resulting paths to different resources. URI dot-segment removal must preserve these distinctions. See [RFC 3986, section 5.2.4](https://www.rfc-editor.org/rfc/rfc3986.html#section-5.2.4).

A loopback fixture answered `GET /base/start` with redirects and recorded the client's next request:

| Location | Expected target | Actual target |
| --- | --- | --- |
| `/target/` | `/target/` | `/target` |
| `/a//b` | `/a//b` | `/a/b` |

The fixture returned a successful body afterwards, so a successful `Send` result does not detect the wrong target. A `..` control case correctly resolved to `/`.

Replace the split-and-drop algorithm with dot-segment removal that preserves empty segments and trailing separators. Test absolute-path and relative locations, `./`, `../`, query-only references, and fragments through a fixture that asserts the exact second request target.

## F05, P2: Content-Length accepts language integer syntax

Source: `lib/Std/Http/BodyFraming.fpas:35`. Shared consumers include `Server.fpas:232`, `Stream.fpas:393`, and `Wire.fpas:252` in the same directory.

The framing selector calls `Std.Parse.TryInt` for `Content-Length`. That parser accepts forms useful in Pascal source but invalid in HTTP framing. Content-Length requires decimal digits. See [RFC 9110, section 8.6](https://www.rfc-editor.org/rfc/rfc9110.html#section-8.6).

A loopback server returned each of these headers with a matching body:

| Header | Client result |
| --- | --- |
| `Content-Length: +2` | Accepted body `ok` |
| `Content-Length: 1_0` | Accepted body `0123456789` |

The client behavior was reproduced. The server uses the same selector, but a server-side exchange was not separately run. Different framing interpretations between peers are a protocol risk; this review does not demonstrate a request-smuggling exploit.

Validate a nonempty ASCII digit sequence and perform bounded conversion. Retain the existing checks for negative values, conflicting lengths, and transfer-encoding conflicts. Add client and server regression cases for signs, separators, radix notation, overflow, and valid boundary values.

## F06, P2: URI scheme validation is bypassed by explicit ports

Source: `lib/Std/Net/Uri.fpas:23`, `:44`, `:67`, and `:118`.

`DefaultPort` validates that the scheme is HTTP or HTTPS. `BuildUri` only calls it when no explicit port was supplied. An explicit port therefore bypasses scheme validation. The port parser also delegates directly to the permissive integer parser.

Public `Parse` results:

| Input | Result |
| --- | --- |
| `ftp://127.0.0.1` | Rejected |
| `ftp://127.0.0.1:21` | Accepted, scheme `ftp`, port 21 |
| `http://127.0.0.1:8_0` | Accepted, port 80 |
| `http://127.0.0.1:+80` | Accepted, port 80 |

This violates the HTTP/HTTPS parser contract in `docs/pascal/std/network/uri.md`. URI port syntax uses decimal digits; see [RFC 3986, section 3.2.3](https://www.rfc-editor.org/rfc/rfc3986.html#section-3.2.3). HTTP connection setup still rejects FTP, so this finding does not establish FTP traffic through `Std.Http`.

Validate the supported scheme independently of default-port selection. Validate port text before numeric conversion. Add a scheme-by-port-presence test matrix and malformed port cases to `tests/stdlib/net/uri_parse_test.fpas`. A small shared protocol decimal parser could address F05 and this port issue without changing `Std.Parse` semantics.

## F07, P2: An SSE limit error retains the oversized fragment

Source: `lib/Std/Http/Sse.fpas:140`, `:200`, and `:249`, especially lines 260 and 269.

`Feed` appends the complete incoming fragment to the stored buffer before `Process` enforces `Maximum`. On error it stores the pre-processing state, including that entire appended buffer. It neither marks the decoder finished nor clears the buffer. Further calls append more bytes before failing again. A caller that keeps feeding after an error can keep growing retained input despite the event limit.

This finding is based on the value flow through `Feed` and `Process`. No heap measurement was made. The current event-size tests verify returned errors, which does not verify retained memory or the subsequent decoder state.

Specify and enforce the state after a decoding error: for example, clear buffered input and make the decoder terminal. Avoid appending an arbitrarily large fragment before checking its events incrementally. A fragment can legitimately contain many small events, so limiting the entire fragment to `MaxEventBytes` would reject valid input. Add repeated-feed-after-error coverage and an allocation/state assertion for retained input. Keep the documented final-event behavior unchanged.
