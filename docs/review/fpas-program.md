# `fpas-program` review follow-up

Classification: persistent program image format and validation. No language change expected. Format compatibility decisions must be explicit.
Status: PROGRAM-01 through PROGRAM-05 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PROGRAM-01 | P1 | `crates/fpas-program/src/image.rs:212`, `src/format/read.rs:40,112` | The 128 MiB limit bounds JSON bytes, not deserialized allocations. Payload is also copied before decoding, so crafted images can cause much larger heap use or OOM. | Borrow the payload slice and impose limits on instructions, locations, functions, constants, and cumulative string bytes during deserialization. | Each resource exactly at and one above its limit; assert early bounded rejection. |
| PROGRAM-02 | P2 | `crates/fpas-program/src/image.rs:125,242,295` | `ProgramImage::new` accepts line/column zero, but decoding rejects it. An image accepted by constructor and encoder can fail its own roundtrip. | Move all location validity into shared image validation used by constructor, encoder, and decoder. | Zero line/column rejected before encode; valid roundtrip remains stable. |
| PROGRAM-03 | P2 | `crates/fpas-program/src/image.rs:137,145,152` | Wire structs accept unknown fields, allowing misspellings or newer fields to be silently discarded despite an explicit format version. | Add strict unknown-field rejection or document and implement an intentional forward-compatibility policy. | Unknown fields at chunk, location, and function levels. |
| PROGRAM-04 | P2 | `crates/fpas-program/src/image.rs:283` | Absolute path rejection uses host-native `Path::is_absolute`; Windows paths can pass on Unix and Unix roots can be mishandled on Windows. | Detect Unix roots, Windows drive roots, and UNC syntax independent of host OS. | Cross-syntax path cases on every host. |
| PROGRAM-05 | P3 | `crates/fpas-program/src/identity.rs:41`, `src/image.rs:270` | Canonical Unit names are documented but only canonicalized for duplicate detection. Case variants produce different deterministic bytes. | Canonicalize at construction or reject non-canonical identities. | Case variants, deterministic encoding, and duplicate detection. |

## Implementation notes

PROGRAM-01 should align resource limits with `fpas-unit` and bundle decoding. PROGRAM-03 is a format-policy decision; document the chosen compatibility behavior in the program-image contributor documentation without describing unimplemented behavior first.

## Implementation record

- PROGRAM-01 borrows the JSON payload directly from the envelope and uses a custom Serde seed to
  stop instructions, locations, constants, functions, and cumulative string data while they are
  decoded. The 128 MiB envelope limit remains; in-memory construction and encoding apply the same
  resource policy.
- PROGRAM-02 validates one-based line and column values through one shared location check used by
  both `ProgramImage::new` and payload decoding. Invalid decoded locations now return
  `ImageError::InvalidLocation` before `SourceLocation` construction can assert.
- PROGRAM-03 rejects unknown chunk, location, and function fields. Format version 1 is explicitly
  strict; schema additions require a new `PROGRAM_FORMAT_VERSION`.
- PROGRAM-04 recognizes Unix roots, Windows drive roots, root-relative Windows paths, and UNC
  paths on every host while preserving relative slash and backslash paths.
- PROGRAM-05 canonicalizes linked Unit identities to ASCII lowercase during image construction.
  Case-only variants therefore retain duplicate detection and produce identical deterministic
  bytes.
- Payload conversion and bounded deserialization were split from the in-memory image model into
  focused `image/payload.rs`, `image/resources.rs`, and `image/resources/collections.rs` modules.
  Internal payload and boundary regressions moved beside their owning modules.
- `docs/pascal/program-structure/compiled-programs.md` documents the implemented image identity,
  validation limits, and strict compatibility policy. FPAS syntax, semantics, and `Std.*` APIs are
  unchanged.

## Verification

- Baseline: `cargo test -p fpas-program --locked` — passed: 17 tests plus doc tests.
- Targeted implementation: `cargo test -p fpas-program --locked` — passed: 31 tests plus doc
  tests.
- Direct dependents: `cargo test -p fpas-program -p fpas-build -p fpas-bundle --locked` — passed.
- `cargo clippy -p fpas-program --all-targets --locked -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.
