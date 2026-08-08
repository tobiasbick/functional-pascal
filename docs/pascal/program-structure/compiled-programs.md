# Compiled program images

Program projects produce a derived `.fpascp` image containing linked executable bytecode, build
identity, reachable Unit identities, and relative source paths for diagnostics. Project manifests
and `.fpas` sources remain authoritative; a source-backed build replaces a missing, stale,
incompatible, or corrupt image.

## Identity and paths

The recorded identity includes the compiler and bytecode versions, source and option hashes, and
the object hash of every reachable Unit in deterministic link order. Unit names are
case-insensitive and are stored in canonical ASCII lowercase, so case-only input differences
produce identical image bytes.

Source paths must be relative. Validation recognizes Unix roots, Windows drive roots, Windows
root-relative paths, and UNC paths independently of the host operating system. Relative paths may
use either slash style because images can be produced and inspected on different hosts.

## Register executable payload

The payload is a deterministic sectioned binary representation of the verified register
executable. All integers use explicit little-endian encoding; no native pointers, host `usize`
values, target triples, object code, or host ABI layouts are stored. The ten sections have a fixed
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
10. the entry function.

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
| Registers per function | 65,535 |
| Call arguments or closure captures | 255 |
| Constants, globals, strings, or source paths | 1,000,000 each |
| Cumulative UTF-8 string data | 64 MiB |
| Linked Units | 1,000,000 |

The same validation is applied before encoding. After decoding, the complete register-executable
verifier checks instruction forms, register windows, numeric IDs, function ranges, control flow,
layouts, source maps, and the entry function before a `RegisterVm` can execute the image.

## Format compatibility

The current sectioned envelope is program format version 10 and register-bytecode version 10. A
producer that changes either wire contract must increment the corresponding version; format and
bytecode changes made for one cutover are versioned together. Readers reject unsupported program or
bytecode versions before execution and `fpas run` tells the user to rebuild from project sources.

Old stack-bytecode images are not migrated. Source-backed project builds replace stale or
incompatible artifacts automatically. A source-less old `.fpascp` cannot run and must be rebuilt on
a checkout that still has its sources.

`.fpascp` is an internal derived format, not a source distribution or package boundary. Rebuild it
from its project sources with the current compiler rather than editing or migrating it by hand.

## See also

- [Projects](projects.md)
- [Command-line interface](cli.md)
- [Units](units.md)
