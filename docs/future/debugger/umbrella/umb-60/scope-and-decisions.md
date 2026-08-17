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

## Frozen launch-owned bound

- `fpas debug` launches and owns an in-process VM. JSONL capabilities
  advertise `attach: false`. JSONL `attach` is `unsupported_capability`.
- DAP initialize advertises `supportsAttach: false`. DAP `attach` fails
  without launching. VS Code contributes only `launch` and rejects
  `request: "attach"` before starting an adapter.
- There is no discovery listener, attach handshake, or debug port on
  `fpas run`. Disconnect ends a launch-owned session. Sources map at launch
  through `--source-root` / image identity, not through attach.
- Native OS debugging is rejected (`UMB-60C`). DAP advertises
  `supportsDisassembleRequest` and `supportsReadMemoryRequest` false.
  `disassemble`, `readMemory`, `writeMemory`, JSONL `disassemble`, and
  `registers` fail without mutating the stopped session.

## `UMB-60A` — local attach

- A successful attach must prove discovery, authorization, disconnect
  ownership, and source mapping without mixing protocol and debuggee bytes.
- That subset requires a debuggee-owned listener and a debugger that connects
  without constructing the VM. Until it exists, attach remains false.
- Begins only at `U60-10`.

## `UMB-60B` — remote sessions

- Begins only after `UMB-60A` can attach locally without protocol ambiguity.
- Authentication, encryption boundary, version negotiation, recovery, and
  privacy limits are required before any remote control path.

## `UMB-60C` — OS-level native debugging

- Rejected. The debuggee is FPAS bytecode in `fpas-vm`, not a native user
  binary. gdb/lldb of the host process would be a second semantic engine.
  One-engine inspection stays at FPAS source/bytecode boundaries.

## Out of scope

- Data breakpoints (`UMB-70`).
- Record/replay (`UMB-80`) and hot reload (`UMB-90`).
- In-call host interruption (rejected by `UMB-50D`).
- Non-stop execution (rejected by `UMB-40D`).
