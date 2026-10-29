# Standard-library namespaces

Status: binding naming and placement policy for contributors. The canonical
units below are implemented.
The [current handbook](../pascal/std/README.md) documents available imports and
behavior; the future allocation table reserves names without providing APIs.

Read this document before adding, moving, splitting or renaming a public
`Std.*` unit, or assigning a unit name in a future plan. Update this document in
the same change when a deliberate design decision changes an allocation.
Keep the [roadmap](../future/std-roadmap.md) for priorities and capability plans.
This specification survives completion of individual APs.

## Naming rules

- `Std` is the reserved standard-library root. Third-party libraries use their
  own root, including libraries that implement an adapter for a standard API.
- Name a domain and then a capability, for example `Std.Net.Http` or
  `Std.Text.Utf8`. Most public paths have one or two segments after `Std`.
  Add another segment when it identifies a useful public subdivision, such as
  `Std.Ai.Acp.Types` or `Std.Encoding.Json.Fields`.
- A focused domain may be a public unit itself, such as `Std.Math`, `Std.Bytes`
  or `Std.Tasks`. A domain prefix does not require an importable parent unit.
- Use the canonical PascalCase spelling from the catalogue, including `Ai`,
  `Io`, `Os`, `Ui`, `Http`, `Utf8`, `OpenAi`, `WebDav`, `Sqlite`, `Draw2D` and
  `Render3D`. Resolution remains case-insensitive. Keep the approved existing
  abbreviations `Fs`, `Proc` and `Cli`.
- Names describe the caller's task. They do not encode the implementation
  language, native dependency, source directory or current maintainer.
- Give each public capability one canonical home. Use concrete domain names
  instead of `Common`, `Utils`, `Helpers` or an extra `Core` layer.
- An import names one unit and retains its explicit alias. Importing a parent
  does not import its children, and sibling units do not inherit visibility.
  Namespace allocation adds no syntax, wildcard imports or implicit exports.
- Keep protocol versions only when they select distinct supported contracts.
  The existing ACP `.V2` and `.Experimental` divisions remain explicit. A
  library release number alone does not justify a versioned namespace.

## Existing public units and migration targets

The current catalogue consists of the intrinsic unit list in
[`std_units/units.rs`](../../crates/fpas-builtins/src/std_units/units.rs) plus the
public source exports in [`stdlib.fpasprj`](../../lib/stdlib.fpasprj).
Private source units and retired type-helper names are not public exports.
The canonical column names the available units. The former-name column records
their previous names; relocated former names are rejected as imports.

| Former unit | Canonical unit | Responsibility |
| --- | --- | --- |
| `Std.Acp` | `Std.Ai.Acp` | ACP connections, events and replies |
| `Std.Acp.Types` | `Std.Ai.Acp.Types` | ACP values and codecs |
| `Std.Acp.Host` | `Std.Ai.Acp.Host` | ACP host adapters |
| `Std.Acp.Experimental` | `Std.Ai.Acp.Experimental` | Explicit ACP feature opt-ins |
| `Std.Acp.V2` | `Std.Ai.Acp.V2` | Distinct ACP v2 contract |
| `Std.Mcp` | `Std.Ai.Mcp` | MCP tool servers |
| `Std.Mcp.Types` | `Std.Ai.Mcp.Types` | MCP definitions, context and results |
| `Std.Ai.OpenAi` | `Std.Ai.OpenAi` | OpenAI-compatible model client |
| `Std.Net` | `Std.Net.Sockets` | Existing TCP/TLS connections and listeners |
| `Std.Http` | `Std.Net.Http` | HTTP client/server and SSE helpers |
| `Std.Net.Uri` | `Std.Net.Uri` | URI parsing and encoding |
| `Std.Server` | `Std.Net.Server` | Server task/listener lifetime and shutdown |
| `Std.Fs` | `Std.Io.Fs` | Files, directories and file-handle I/O |
| `Std.Path` | `Std.Io.Path` | Pure filesystem-path manipulation |
| `Std.Args` | `Std.Os.Args` | Process arguments |
| `Std.Env` | `Std.Os.Env` | Process environment |
| `Std.Proc` | `Std.Os.Proc` | Child processes and their pipe operations |
| `Std.Errors` | `Std.Os.Errors` | Shared native host error causes |
| `Std.Cli` | `Std.Cli` | Command-line schemas, parsing and help |
| `Std.Console` | `Std.Console` | Terminal text, cells, screen and input |
| `Std.Tui` | `Std.Ui.Terminal` | Existing terminal MVU interface |
| `Std.Conv` | `Std.Text.Format` and `Std.Math` | Split text formatting from numeric conversion |
| `Std.Parse` | `Std.Text.Parse` | Scalar text parsing |
| `Std.Net.Utf8` | `Std.Text.Utf8` | UTF-8 encoding and decoding |
| `Std.Encoding` | `Std.Encoding` | Existing Base64, Base64url and hex codecs |
| `Std.Json` | `Std.Encoding.Json` | JSON values, parsing and serialization |
| `Std.Json.Fields` | `Std.Encoding.Json.Fields` | Typed JSON field access |
| `Std.Toml` | `Std.Encoding.Toml` | TOML values, parsing and serialization |
| `Std.Toml.Fields` | `Std.Encoding.Toml.Fields` | Typed TOML field access |
| `Std.Math` | `Std.Math` | Scalar mathematics and numeric conversion |
| `Std.Bits` | `Std.Math.Bits` | Integer bit operations |
| `Std.Random` | `Std.Math.Random` | Pseudorandom sequences |
| `Std.Time` | `Std.Time` | Clocks, timestamps and waiting |
| `Std.Calendar` | `Std.Time.Calendar` | Gregorian dates and UTC calendar arithmetic |
| `Std.Crypto` | `Std.Crypto` | Cryptographic operations and secure randomness |
| `Std.Tasks` | `Std.Tasks` | Tasks, channels, cancellation and selection |
| `Std.Test` | `Std.Test` | Test assertions and test facilities |
| `Std.Version` | `Std.Version` | Compiler and library version information |

The former `Std.Conv.IntToStr`, `RealToStr`, `BoolToStr` and `IntToHex` belong to
`Std.Text.Format` with the same routine names. The former `Std.Conv.IntToReal`
belongs to `Std.Math.IntToReal`. Signatures, rounding and error behavior are
preserved. A caller
using both parts imports both destination units.

All other entries preserve their public members and behavior while changing
the owning unit where listed. Public nominal identities, generated declarations,
diagnostic operation labels and cached interfaces must use the new canonical
path together. Rebuild affected artifacts; do not publish forwarding units or
legacy aliases. Internal identifiers need changes only where public identity or
clear ownership requires them.

The `Std.Tui` move changes names only. A common UI tree and a graphical host
remain work for the [shared UI plan](../future/shared-ui.md). Moving `Std.Net`
does not add UDP, split TCP from TLS or change connection ownership.

## Allocations for future capabilities

The following homes are reserved if their capabilities are introduced. These
are naming decisions, not available APIs, delivery promises or permission to
implement features. Use a concrete requirement to select
a capability and design its types, operations, limits and lifecycle separately.
Create only the units that a delivered interface needs.

| Domain | Canonical homes | Placement rule |
| --- | --- | --- |
| Bytes and binary values | `Std.Bytes`, `Std.Bytes.Binary` | General byte storage; multi-byte number encoding and explicit byte order |
| I/O | `Std.Io.Fs`, `Std.Io.Path`, `Std.Io.Streams`, `Std.Io.Pipes` | Filesystem operations, paths, general stream contracts and standalone pipes |
| Host services | `Std.Os.Args`, `Std.Os.Env`, `Std.Os.Proc`, `Std.Os.Errors`, `Std.Os.Signals`, `Std.Os.Hardware` | Portable process and host facilities, native causes and hardware information |
| Platform-specific access | `Std.Platform.Windows`, `Std.Platform.Unix` and focused children | Explicitly platform-specific facilities that cannot keep a portable contract |
| Command lines | `Std.Cli` | User-facing option schemas, help and validation; raw arguments stay in `Std.Os.Args` |
| Terminal primitives | `Std.Console`, optional `Std.Console.Screen` and `Std.Console.Input` | Text streams, cell rendering and terminal events without a widget framework |
| Text | `Std.Text.Parse`, `Std.Text.Format`, `Std.Text.Utf8`, `Std.Text.Unicode`, `Std.Text.Regex` | Text representation, scalar parsing/formatting and Unicode-aware processing |
| Data formats | `Std.Encoding`, `Std.Encoding.Json`, `Std.Encoding.Toml`, `Std.Encoding.Xml`, `Std.Encoding.Yaml`, `Std.Encoding.Csv`, `Std.Encoding.Cbor`, `Std.Encoding.MessagePack` | Generic structured formats and binary/text codecs; field helpers belong beneath their format |
| Codec subdivisions | `Std.Encoding.Base64`, `Std.Encoding.Hex` | Use only if a separate API change extracts the existing codecs; the current codecs remain in `Std.Encoding` |
| Numbers and geometry | `Std.Math`, `Std.Math.Bits`, `Std.Math.Random`, `Std.Math.Statistics`, `Std.Math.LinearAlgebra`, `Std.Math.Geometry` | Computation independent of a renderer, audio device or network protocol |
| Time | `Std.Time`, `Std.Time.Calendar`, `Std.Time.Zones` | Clock/duration operations, civil calendars and time-zone rules |
| Localization | `Std.Globalization`, `Std.Globalization.Locale`, `Std.Globalization.Collation`, `Std.Globalization.Messages` | Locale-sensitive presentation, collation and message localization |
| Collections | `Std.Collections` and concrete container children | Additional containers; built-in array/dictionary operations remain native |
| Algorithms | `Std.Algorithms` and focused children | Reusable algorithms without a better domain owner; preserve existing native collection operations |
| Concurrency | `Std.Tasks` and justified children | Task ownership, channels, cancellation, selection and synchronization |
| Network foundations | `Std.Net.Sockets`, `Std.Net.Dns`, `Std.Net.Uri`, `Std.Net.Server` | TCP/TLS and future datagram endpoints, name resolution, URIs and server lifetime |
| Network protocols | `Std.Net.Http`, `Std.Net.WebSocket`, `Std.Net.WebDav`, `Std.Net.Ftp`, `Std.Net.Sftp`, `Std.Net.Ssh`, `Std.Net.Mqtt`, `Std.Net.Rpc` | Each protocol owns its client/server operations; FTPS belongs with FTP, SFTP with SSH-based transfer |
| Web applications | `Std.Web.Html`, `Std.Web.Templates`, `Std.Web.Routing` | Web content and application composition over HTTP |
| AI and agents | `Std.Ai.Acp`, `Std.Ai.Mcp`, `Std.Ai.OpenAi` and concrete provider/protocol children | Model services, agent protocols and their host adapters |
| Cryptography | `Std.Crypto` and focused children | Secure randomness, hashes, encryption, password hashing and signatures |
| Security policy | `Std.Security` and focused children | Credential storage, authentication and authorization policy; cryptographic mechanisms stay in `Std.Crypto` |
| Databases | `Std.Db.Sqlite` and concrete database children | Database-specific connections, statements, transactions and backup |
| Archives | `Std.Archive.Zip`, `Std.Archive.Tar` | Named entries, archive metadata and extraction |
| Compression | `Std.Compress.Gzip`, `Std.Compress.Zstd` and concrete codecs | Byte-stream compression without archive-entry semantics |
| Images and graphics | `Std.Graphics.Image`, `Std.Graphics.Draw2D`, `Std.Graphics.Render3D`, `Std.Graphics.Gpu` | Image data/codecs, drawing, 3D rendering and explicit GPU resources |
| Windows and devices | `Std.Window`, `Std.Input` | Native window lifecycle and non-terminal keyboard, pointer, touch or controller input |
| Audio | `Std.Audio`, `Std.Audio.Devices`, `Std.Audio.Mixer`, `Std.Audio.Codecs` | Audio samples, device I/O, mixing and audio formats |
| Video | `Std.Video` and focused children | Time-based video frames and codecs; still-image operations belong to graphics |
| User interfaces | `Std.Ui`, `Std.Ui.Terminal`, `Std.Ui.Graphical` | Shared semantic UI and its terminal/graphical hosts |
| Diagnostics | `Std.Diagnostics.Log`, `Std.Diagnostics.Metrics`, `Std.Diagnostics.Trace`, `Std.Diagnostics.Health` | Structured evidence and health independent of presentation or transport |
| Native interoperation | `Std.Ffi` and concrete ABI children | Native library loading and calls, subject to the separate FFI design |
| Runtime services | `Std.Runtime` and focused children | Explicit public execution/runtime services, including future FPAS module loading |
| Testing and versioning | `Std.Test`, `Std.Version` | Test facilities and FPAS version information |

Optional subdivisions require a deliberate interface change and one canonical
home; they do not duplicate parent operations.

## Boundaries between domains

- A byte buffer belongs to `Std.Bytes` even when most callers perform I/O.
  Pixel, vertex/index and sample buffers belong to graphics or audio. They may
  share internal storage without exposing native pointers or a C ABI.
- `Std.Io` owns transfer and filesystem contracts. `Std.Os` owns process and
  host services. Child-process pipe functions remain with `Std.Os.Proc` until
  a concrete shared pipe interface warrants a separate design.
- UTF-8 belongs to text. Base64 and hex represent arbitrary bytes and belong
  to encoding. JSON/XML are generic data formats. HTML application facilities
  belong to web, and PNG or WAV codecs belong to graphics or audio.
- `Std.Net` owns protocols regardless of payload. A WebDAV client belongs to
  `Std.Net.WebDav`; a future filesystem adapter over that client belongs to
  `Std.Io` and depends on it. SSH and SFTP do not become FTP variants.
- `Std.Ai` may depend on HTTP, sockets, JSON and bytes. Those dependencies do
  not move the AI interface into `Std.Net` or make lower layers depend on AI.
  Share generic RPC machinery only when it has an independent public use.
- Graphics and audio can be used without UI. Windows and device input can be
  used by a renderer without widgets. `Std.Ui` composes these services; it does
  not own their general-purpose interfaces. Terminal cell/input operations
  stay with `Std.Console`, below the terminal UI.
- Pure vector, matrix and geometry calculations belong to `Std.Math`.
  Renderer-specific vertex layouts, shaders and GPU resources belong to
  `Std.Graphics`. Neither exposes the native backend as the default public API.
- Secure randomness belongs to `Std.Crypto`; reproducible pseudorandom
  sequences belong to `Std.Math.Random`. Network TLS remains a transport
  capability. A future shared certificate API needs an explicit allocation.
- Domain errors remain with their owning unit. `Std.Os.Errors.HostError`
  carries shared native causes; it is not a universal enum for parsing,
  protocols, task failures or all operational errors.
- Locale-neutral parsing/formatting stays under `Std.Text`. Locale-sensitive
  rules belong to `Std.Globalization`; date arithmetic and time zones stay
  under `Std.Time`.
- Distributed applications compose networking, storage, cryptography and
  diagnostics. The distributed-node plan does not automatically require a
  public `Std.Distributed` framework. Allocate any reusable interface through
  the extension procedure below.

## Public units and internal files

Public units exist for coherent caller tasks, not for every implementation
file. Keep private source units private in the standard-library manifest.
Place their names beneath the owning public family when practical, without
automatically exporting them or requiring matching public subunits.

Organize source directories by domain. Unit declarations and the export
catalogue determine identity; a directory move alone cannot rename an API.
Generated declarations, intrinsic registrations and source-defined exports
follow the same public naming policy. A native implementation does not receive
a shorter or privileged namespace.

Namespace ownership does not determine optional dependencies, installed
packages, host capabilities or runner selection. Those delivery decisions
belong to the corresponding capability plan. Refer to this catalogue rather
than copying it into each plan or handbook.

## Adding a capability not covered here

1. State the caller's task, inputs/results and resource ownership. Identify
   the current public units it would use and the nearest existing domain.
2. Prefer that domain when it describes the task without changing its meaning.
   Add a child when it has a coherent interface that callers use independently.
3. Propose a new root only when no existing domain fits. Describe the overlap,
   dependency direction and why an existing domain would mislead callers.
   A vendor name, source dependency or single implementation file is not enough.
4. Record the chosen canonical path and boundary here, with the feature's
   existing plan or new AP. If revising an agreed allocation, record the
   replacement and affected consumers in that work package.
5. Keep feature approval separate from naming. A reserved home does not settle
   syntax, semantics, signatures, platform coverage or implementation scope.

Before delivering a new or moved unit, check its catalogue entry, export list,
type ownership, diagnostics, generated declarations, CLI/editor lookup,
documentation and affected consumers. Apply the AP's migration and regression
checks. Update the current handbook only with the implemented API.
