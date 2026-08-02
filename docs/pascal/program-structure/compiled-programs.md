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

## Validation and resource limits

The envelope hash is checked before its JSON payload is decoded. Decoding borrows the payload from
the input image and rejects a resource as soon as it exceeds these limits:

| Resource | Maximum |
|---|---:|
| Encoded payload | 128 MiB |
| Instructions | 1,048,576 |
| Source locations | 1,048,576 |
| Functions | 65,535 |
| Constants | 65,535 |
| Cumulative UTF-8 string data | 16 MiB |

The same in-memory validation is applied before encoding. Locations require one-based line and
column values, source identifiers must reference the image's path table, constants must be
persistent runtime-independent values, and the complete executable bytecode validator must pass.

## Format compatibility

The current envelope is format version 1. Its JSON payload is strict: unknown fields in the chunk,
source-location, or function records are rejected instead of silently discarded. A producer that
changes this schema must use a new `PROGRAM_FORMAT_VERSION`; readers reject unsupported envelope
versions and incompatible bytecode versions explicitly.

`.fpascp` is an internal derived format, not a source distribution or package boundary. Rebuild it
from its project sources with the current compiler rather than editing or migrating it by hand.

## See also

- [Projects](projects.md)
- [Command-line interface](cli.md)
- [Units](units.md)
