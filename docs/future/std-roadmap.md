# Standard library roadmap

Status: capability priorities; each capability retains the status of its own plan.

This is the central roadmap for standard-library expansion. The binding
[namespace specification](../specs/std-namespaces.md) owns canonical homes,
domain boundaries and rules for capabilities not yet anticipated here.
The [standard library reference](../pascal/std/README.md) describes available
APIs. A reserved namespace is not an implemented feature or a delivery promise.

## Direction

- Keep public units focused on a caller's task and grouped by domain.
- Use the namespace specification when adding or moving a capability. Update
  it with any deliberate allocation change rather than making a competing map.
- Keep pure data and computation usable independently of host resources.
- Keep process, filesystem, clock and device ownership explicit in hosted APIs.
- Use one byte representation across consumers. Pixel, vertex and audio formats
  may share storage while retaining their domain-specific contracts.
- General raw pointers and memory-management APIs need their own FFI/runtime
  design; a managed byte buffer does not imply those capabilities.
- Retain a broad bundled library with consistent FPAS-facing contracts. Select
  native dependencies and optional delivery requirements in the feature plans.

## Selected order

1. Implement [AP54's byte-buffer plan](improve-syntax/ap54-byte-buffers/README.md)
   under `Std.Bytes` when authorized, including UTF-8 integration and migration
   of existing binary APIs. Its contract is complete: copy-on-write values,
   explicit mutation, content equality, strict UTF-8, a 256 MiB maximum, bounded
   slice retention, automatic release and full artifact/debugger support. The
   broader alternatives stay in
   [FPAS 2 section 22](fpas2-language-directions.md#22-packed-numeric-data-and-binary-buffers).
2. Continue selected FPAS 2 work, then graphics/multimedia and the
   [shared terminal/graphical UI](shared-ui.md) in their recorded order.
   Other capability plans keep their actual prerequisites.

Streams, UDP, additional protocols, graphics, audio and shared UI retain their
separate feature plans.
The AP index is the completion tracker; this roadmap does not duplicate its
work-package checklists.

## Existing capabilities

FPAS already supplies console and terminal UI, text and numeric helpers,
structured formats, tasks/channels, file handles, processes, TCP/TLS, HTTP,
secure randomness and AI protocol integrations. Their current names and
contracts are documented in the handbook. In particular:

- [Tasks](../pascal/std/concurrency/task.md) include bounded channels,
  cancellation, selection, groups and supervision.
- [Server lifetime](../pascal/std/network/server.md) owns task/listener shutdown
  and explicit process escalation.
- [Filesystem I/O](../pascal/std/host/fs-streams.md) includes incremental bytes
  and strict UTF-8 lines. Existing text-file operations use UTF-8.
- [Cryptography](../pascal/std/cryptography/crypto.md) supplies operating-system
  random bytes and secure integer sampling.

The specification's [canonical-unit catalogue](../specs/std-namespaces.md#current-public-units)
owns their current names and responsibilities.

## Future capability plans

| Concern | Owning plan | Scope and prerequisites |
| --- | --- | --- |
| Byte buffers | [AP54: Native byte buffers and UTF-8 integration](improve-syntax/ap54-byte-buffers/README.md) | Contract complete; implement one shared representation and migrate existing binary APIs when authorized; typed media storage follows separately |
| SQLite | [SQLite storage](networked-applications/sqlite.md) | Concrete database access, transactions and backup |
| Cryptography | [Cryptography](networked-applications/cryptography.md) | Password hashing, authentication primitives and signatures |
| WebSocket | [WebSocket transport](networked-applications/websocket.md) | Bounded messages over TCP/TLS and HTTP |
| UDP | [UDP sockets](networked-applications/udp.md) | Datagram operations after a concrete use case |
| WebDAV | [WebDAV](webdav.md) | HTTP-based remote resource operations |
| FTP/FTPS | [FTP](ftp.md) | Control/data connections and transfer lifecycle |
| Diagnostics | [Observability](networked-applications/observability.md) | Structured logs, metrics, traces and health |
| Distributed applications | [Distributed nodes](networked-applications/distributed-nodes.md) | Compose identity, storage, transport and conflict handling |
| Graphics and audio | [Graphics](graphics.md) and [multimedia alternatives](graphics-alternative.md) | Rendering, media, windows, input and native delivery after relevant FPAS 2 foundations |
| Shared UI | [Terminal and graphical UI](shared-ui.md) | Common UI and both hosts during graphics work |
| Hardware information | [Hardware information](hardware-information.md) | CPU/memory information; idea only |

The [networked application platform](networked-applications/README.md) owns
cross-capability dependencies for storage, security, transports and operations.
Further areas, including localization, additional formats, archives,
compression, collections, algorithms, video, FFI and platform services, have
homes in the namespace specification but no implementation work implied here.
Add a concrete feature plan when selecting one of them.

## Implementation checklist

For every new or moved public standard unit:

- confirm its canonical home and domain boundary in the namespace specification;
- record the selected contract, prerequisites, migration and acceptance in its AP;
- update the unit handbook and standard-library index with delivered behavior;
- update intrinsic registration, compiler/VM wiring and source exports as needed;
- regenerate intrinsic editor declarations and verify CLI/editor lookup;
- migrate affected consumers, examples, fixtures, generators and agent guidance;
- reuse suitable tests and add coverage for distinct observable gaps;
- verify public/private visibility and dependent compiled-artifact identities.

Use the [shared implementation touchpoints](../pascal/std/README.md#shared-implementation-touchpoints)
and [AP workflow](improve-syntax/README.md) for the owning paths and checks.

## Open decisions

- General stream abstractions and worker-releasing I/O, coordinated with
  [FPAS 2 host I/O](fpas2-language-directions.md#14-host-io-that-releases-vm-workers).
- Future resource/capability contracts and platform-specific behavior, chosen
  in the feature plans rather than inferred from a namespace.
- Optional native dependencies, runners and packaging for multimedia and other
  hosted extensions.

The naming policy is binding. These runtime and delivery questions remain open
and require their own design and implementation decisions.
