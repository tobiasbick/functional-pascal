# Compiled program images

Program projects produce a derived `.fpascp` image containing linked executable bytecode, build
identity, reachable Unit identities, and relative source paths for diagnostics. Project manifests
and `.fpas` sources remain authoritative; a source-backed build replaces a missing, stale,
incompatible, or corrupt image. Immediately before reuse or publication, the build revalidates the
complete source snapshot while holding the image's publication lock. If any source changed during
the build, that builder fails instead of replacing an image produced from newer sources.

## Identity and paths

The recorded identity includes the contract policy, compiler and bytecode versions, source and option hashes, and
the object hash of every reachable Unit in deterministic link order. Unit names are
case-insensitive and are stored in canonical ASCII lowercase, so case-only input differences
produce identical image bytes.

Header flag bit 0 records disabled contracts; zero means contracts enabled.
Unknown flag bits are rejected. The policy also participates in unit and program
option identities, preventing cache reuse across policies. Direct execution
retains the image's policy; see [CLI contract policy](cli.md#contract-policy).

Source paths must be relative. Validation recognizes Unix roots, Windows drive roots, Windows
root-relative paths, and UNC paths independently of the host operating system. Relative paths may
use either slash style because images can be produced and inspected on different hosts.

Artifact reuse requires every source path retained by the image to have the same content digest in
the current build snapshot. The build matches these digests by portable path because the image has
already replaced compiler-local source identifiers with concrete paths. Reordering the wider project
source list therefore remains reusable when all retained path-to-content bindings are unchanged.

### Source roots for debugging

CLI builds and project debugging derive paths from the same canonical source
root: the smallest directory containing the project directory, main program and
resolved source-unit dependencies. Merely available, unselected units do not
widen that root. Paths use forward slashes and contain no upward traversal.
`fpas build` prints `Debug source root: ...` for both new and reused program images.

For sibling `app` and `shared` projects, retained paths can be
`app/src/main.fpas` and `shared/src/main.fpas`. Debug their image with
`fpas debug app/app.fpascp --source-root .` from that common parent. The image
may be stored elsewhere; `--source-root` identifies the source tree, not the
image directory. A relocated tree with identical relative paths and source bytes
is accepted. Source hashes and canonical containment are checked before execution.

If selected sources span different drive roots, normal builds retain their
existing labels and report that no common debug source root exists. Use the
project target, or stage the sources below one root and rebuild for image
debugging. Rebuild older images with upward-relative source paths; their source
table is refreshed without changing the program format.

## Register executable payload

The payload is a deterministic sectioned binary representation of the verified register
executable. All integers use explicit little-endian encoding; no native pointers, host `usize`
values, target triples, object code, or host ABI layouts are stored. The twelve sections have a fixed
order:

1. strings;
2. constants;
3. relative source paths;
4. globals;
5. record layouts;
6. enum layouts and variants;
7. function metadata;
8. packed 64-bit register instructions;
9. sparse source-map runs;
10. the entry function;
11. portable debugger types;
12. source-visible distinct type names.

The directory requires every section exactly once, in canonical order, with contiguous ranges and
no trailing bytes. This keeps byte output stable for the same identity and executable.

## Validation and resource limits

The envelope hash is checked before any executable section is decoded. Decoding rejects unknown
flags and tags, duplicate or reordered sections, invalid UTF-8, truncation, noncanonical booleans or
enum tags, range overflow, trailing data, and resources above the configured limits. Important
limits are:

| Resource | Maximum |
|---|---:|
| Encoded payload | 512 MiB |
| Instructions | 16,000,000 |
| Instructions per function | 4,000,000 |
| Sparse source-map runs | 4,000,000 |
| Functions | 65,536 |
| Record or enum layouts | 65,536 each |
| Fields per record or enum variant | 65,535 |
| Semantic type nodes per error key or variant | 65,535, depth below 64 |
| Generated error metadata per link expansion | 1,000,000 nodes; 64 MiB copied text |
| Registers per function | 65,535 |
| Call arguments or closure captures | 255 |
| Constants, globals, strings, or source paths | 1,000,000 each |
| Cumulative UTF-8 string data | 64 MiB |
| Linked Units | 1,000,000 |

The same validation is applied before encoding. After decoding, the complete executable
verifier checks instruction forms, register windows, numeric IDs, function ranges, control flow,
layouts, source maps, and the entry function before the VM can execute the image.

## Format validation

The sectioned envelope uses program format version 23 and register-bytecode version 28.
Relocatable unit objects use schema version 17 and compiled-unit envelopes use
version 22. Error layouts retain complete nominal instance keys, literal labels,
concrete semantic payload types and typed Cause references independently of optional
debugger information. Keys retain all generic arguments; ordinary enum layouts keep
their existing erasure. Bounded postfix type identities avoid recursive wire decoding.

Unit objects retain error declaration schemas, their source provenance and hidden
generic constructor requirements. Linking resolves concrete requests to shared
noncapturing functions using the existing instruction set. Error values reuse shared
enum storage. A shared expansion budget rejects amplified schemas before copying
their fields or labels. Error values use
automatic iterative display and stack-safe aggregate destruction;
the Value representation remains 16 bytes. Native error declarations and
Error-constrained generic routines emit this metadata. Hidden constructor
parameters and bound callable environments preserve the source arity and labels.
Generated linkage names are separate from their defining display names. Hidden
callable captures are immutable, type-checked functions and remain absent from
normal debugger scopes. Existing Std Result signatures retain their current types.

Bytecode version 28 permits auxiliary flag 1 on Result/Option construction
to consume a source register proven dead by the compiler.
Validation succeeds before the source is removed; failed operations preserve it
for diagnostics and inspection. Live sources use the ordinary copying form.
Pattern payload extraction keeps its source for subsequent pattern arms.
Single-argument intrinsics borrow the argument's existing register without an
intermediate copy window. Their error behavior and ownership checks are unchanged.
Older images and unit objects require rebuilding.
Source-backed caches rebuild through their bytecode identity checks; standalone
artifacts with the earlier ABI are rejected before execution. Instruction
encoding, envelope layouts and the 16-byte value representation are unchanged.
These formats are versioned independently: a producer that changes a wire
contract or executable ABI must increment the corresponding version. The version
numbers need not match.
Readers reject unsupported versions before execution, and
`fpas run` tells the user to rebuild from project sources. Source-backed project builds replace
incompatible artifacts automatically. A source-less incompatible `.fpascp` cannot run and must be
rebuilt from its sources.

`.fpascp` is an internal derived format, not a source distribution or package boundary. Build it
from project sources with the compiler rather than editing it by hand.

## See also

- [Runtime ownership for Rust hosts](runtime-ownership.md)
- [Projects](projects.md)
- [Command-line interface](cli.md)
- [Units](units.md)
