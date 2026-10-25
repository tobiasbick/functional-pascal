# AP50.5: Inbound requests and host adapters

Package: [AP50](README.md). Status: planned; implementation not started.

## Prerequisites

[AP50.2](02-process-support.md), [AP50.3](03-connection-runtime.md) and
[AP50.4](04-v1-client.md) are delivered. AP50.1 provides typed inbound requests,
responses, elicitation schemas and permission outcomes.

## Contract and scope

Implement all stable incoming requests and notifications in
[protocol-coverage.md](protocol-coverage.md). Every request reaches application
code as typed data plus a Responder. Receiving it grants no host authority.
Capabilities are configured explicitly, based on providers the application can
actually serve. The application can answer directly or invoke a built-in
`Std.Acp.Host` operation after deciding to permit the action.

`Host.ReadTextFile`, `WriteTextFile`, `CreateTerminal`, `TerminalOutput`,
`WaitForTerminalExit`, `KillTerminal` and `ReleaseTerminal` accept the original
Responder and corresponding typed request. They validate method, connection,
session and active request identity before admitting effects. They return the
corresponding typed response or AcpError, without consuming the responder.
Host errors are mapped
to an explicit protocol error by application code. A sample dispatcher shows
this route without auto-approving requests or converting typed errors to strings.

## Permission and elicitation

Present permission options unchanged, including allow/reject once/always forms.
Validate a selected option ID before replying. The library never implements
allow-always as a permanent capability grant. A cancelled/expired responder
cannot authorize a later operation. Session cancellation sends the required
cancelled outcome for all pending permissions even if the UI has not answered.

Form elicitation exposes the full restricted schema, labels/defaults, required
fields and supported primitive/enum choices. Supply a validator for form
responses against that schema. The application presents an editable result and
explicit accept/decline/cancel controls; invalid submission is not sent.
URL elicitation exposes the requesting identity, destination host, message and
elicitation ID. The host obtains consent before opening it, and completion
notifications correlate only with the receiving connection/user interaction.
No implicit browser navigation or credential collection belongs to the library.

## Files and terminals

The disk adapter receives an explicit request for one absolute path; it is not
a workspace sandbox. Applications may enforce a root policy or answer from an
editor buffer instead. Do not use lexical path normalization as a security claim.
ReadTextFile uses one-based line/limit semantics, preserves text terminators
according to the pinned protocol, and returns an empty range after EOF.
WriteTextFile creates a missing file in an existing parent and reports failures
without a fabricated success. Use existing Std.Fs logic for validated UTF-8 and
file lifecycle. Bound response construction by connection limits.

The terminal adapter creates owned processes using the common process core.
Command, args, env and cwd remain separate fields. Return terminal identity
after successful launch without waiting for exit. Drain stdout and stderr while
retaining their arrival-observed combined output; do not promise an ordering the
OS does not provide across separate pipes. Bound the retained suffix by bytes
and truncate only at UTF-8 scalar boundaries. Invalid UTF-8 becomes replacement
characters under a documented decoding policy, with byte limits still enforced.

Keep a terminal registry per connection with session association. Output and
exact exit status remain queryable after exit and kill. WaitForTerminalExit
suspends the caller, not connection dispatch. Release terminates remaining work,
reaps the process, invalidates aliases and frees the registry entry. Repeated
release is a typed closed error. Connection close releases every terminal;
session close releases that session's terminals after applicable final output.
Previously emitted detached output may remain in application-owned UI state.

## Interactive authentication

`Host.RunTerminalAuthentication(Launch, AuthMethodTerminal, TimeoutMillis,
CancellationToken)` merges the method's args/env with the original base launch
configuration and delegates to AP50.2. Return an explicit success/failure result;
zero exit alone is success. On success the application closes/reconnects and
initializes a new ACP connection with the base launch config. No existing
session or prompt is automatically replayed. Hosts without an interactive
terminal return a typed unsupported outcome before launching anything.

## Affected files and implementation owners

| Action | Path | Purpose |
| --- | --- | --- |
| Create | `crates/fpas-acp/src/interaction/permission.rs`, `elicitation.rs`, `authentication.rs` | Validation, cancellation and schema/option decisions |
| Create | `crates/fpas-vm/src/vm/hosted/acp/host/` | Files, terminals, interactive login and registry cleanup |
| Modify | `crates/fpas-vm/src/vm/hosted/proc/`, `hosted/fs/` | Reuse existing production process/file operations |
| Create/modify | `crates/fpas-sema/src/std_registry/loaded/acp/host.rs`, native names/catalog | Host operations and precise typed responses |
| Create | `tests/stdlib/acp/host_requests_test.fpas`, `elicitation_test.fpas` | Application responders, policy refusal and form validation |
| Create | `crates/fpas-cli/tests/acp/host.rs`, `crates/fpas-vm/src/vm/hosted/acp/tests/host.rs` | Distinct real-process, file and lifecycle behavior |
| Create | `docs/pascal/std/ai/acp/host.md`, `authentication.md` | Opt-in effects, errors and application responsibilities |
| Generate | `lib/api/Std/Acp.Host.fpas` | Accurate public host declarations |

If existing fs/proc entry points are too tightly coupled to intrinsic stack
arguments, extract their native operation once. Do not duplicate them in ACP.

## Acceptance and verification

- A real peer waits for a permission/form/URL response while another session
  progresses. Decline and cancel cause no file/process/browser effect.
- Missing, unknown and stale option/elicitation IDs fail at the proper boundary.
  Cover every restricted form type, defaults, required fields and all outcomes.
- Disk reads cover first/last/beyond-end lines, omitted/zero/invalid limits under
  schema rules, Unicode and inaccessible files. A custom responder returns an
  unsaved buffer, proving the library does not force a disk read.
- Terminal create/output/wait/kill/release cover output truncation at split UTF-8,
  output after exit, nonzero exit, Unix signals, wrong-session terminal IDs,
  cancellation of wait and close while running. No descendant survives release.
- Terminal authentication preserves base arguments, cwd and environment,
  applies overrides exactly once, and never consumes the ACP protocol pipes.
  Unsupported terminal hosts, cancellation and nonzero exits do not reconnect.
- Try to use the same Responder twice and respond after cancellation. The peer
  observes exactly one response and no authorized effect from a stale answer.

Run owning interaction/host/process tests and public FPAS cases. Each work
package retains its current handbook/examples and required checks.
