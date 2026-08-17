# UMB-60 scope and decisions

## Shared invariants

1. JSONL and DAP protocol bytes must remain distinguishable from debuggee
   I/O. A mixed unframed stream is forbidden.
2. Attach does not create a second debugger engine. Adapters map events onto
   the shared VM session.
3. A rejection changes no worker, scheduler result, waiter, stop generation,
   or adapter state.
4. JSONL and DAP call the same session or host operation. VS Code maps to
   that adapter behavior.
5. No FPAS syntax, semantics, or language documentation changes are in scope.
6. Unauthenticated discovery and default exposure of host paths, environment
   data, or sources are forbidden.

## Current ownership inventory

These are inventory facts for `U60-00`, not acceptance of later children.

- `fpas debug` launches and owns the debuggee. JSONL capabilities advertise
  `attach: false`. DAP initialize does not advertise attach; unsupported
  requests fail explicitly.
- VS Code documents launch-owned debugging; attach remains unsupported.
- Protocol stdin/stdout remain the launch-owned JSONL or DAP transport proven
  by `UMB-50`. There is no discovery listener, attach handshake, or remote
  authentication surface.
- Disconnect currently ends a launch-owned session. Attach-specific ownership
  of an independently running VM or bundle is not implemented.
- Native OS debugging would be a second semantic engine unless it only
  supplies the existing VM session. `UMB-60C` owns that go/no-go.

## `UMB-60A` — local attach

- Begins only after `U60-00` records the current launch-owned bound.
- A successful attach must prove discovery, authorization, disconnect
  ownership, and source mapping without mixing protocol and debuggee bytes.
- Until that subset is proven, attach remains false.

## `UMB-60B` — remote sessions

- Begins only after `UMB-60A` can attach locally without protocol ambiguity.
- Authentication, encryption boundary, version negotiation, recovery, and
  privacy limits are required before any remote control path.

## `UMB-60C` — OS-level native debugging

- Go/no-go based on the actual runtime and bundle model.
- A second semantic debugger engine is forbidden. Native tooling may only
  exist if it reuses the shared VM session.

## Out of scope

- Data breakpoints (`UMB-70`).
- Record/replay (`UMB-80`) and hot reload (`UMB-90`).
- In-call host interruption (rejected by `UMB-50D`).
- Non-stop execution (rejected by `UMB-40D`).
