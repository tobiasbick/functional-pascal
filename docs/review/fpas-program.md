# `fpas-program` review follow-up

Classification: persistent program image format and validation. No language change expected. Format compatibility decisions must be explicit.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| PROGRAM-01 | P1 | `crates/fpas-program/src/image.rs:212`, `src/format/read.rs:40,112` | The 128 MiB limit bounds JSON bytes, not deserialized allocations. Payload is also copied before decoding, so crafted images can cause much larger heap use or OOM. | Borrow the payload slice and impose limits on instructions, locations, functions, constants, and cumulative string bytes during deserialization. | Each resource exactly at and one above its limit; assert early bounded rejection. |
| PROGRAM-02 | P2 | `crates/fpas-program/src/image.rs:125,242,295` | `ProgramImage::new` accepts line/column zero, but decoding rejects it. An image accepted by constructor and encoder can fail its own roundtrip. | Move all location validity into shared image validation used by constructor, encoder, and decoder. | Zero line/column rejected before encode; valid roundtrip remains stable. |
| PROGRAM-03 | P2 | `crates/fpas-program/src/image.rs:137,145,152` | Wire structs accept unknown fields, allowing misspellings or newer fields to be silently discarded despite an explicit format version. | Add strict unknown-field rejection or document and implement an intentional forward-compatibility policy. | Unknown fields at chunk, location, and function levels. |
| PROGRAM-04 | P2 | `crates/fpas-program/src/image.rs:283` | Absolute path rejection uses host-native `Path::is_absolute`; Windows paths can pass on Unix and Unix roots can be mishandled on Windows. | Detect Unix roots, Windows drive roots, and UNC syntax independent of host OS. | Cross-syntax path cases on every host. |
| PROGRAM-05 | P3 | `crates/fpas-program/src/identity.rs:41`, `src/image.rs:270` | Canonical Unit names are documented but only canonicalized for duplicate detection. Case variants produce different deterministic bytes. | Canonicalize at construction or reject non-canonical identities. | Case variants, deterministic encoding, and duplicate detection. |

## Implementation notes

PROGRAM-01 should align resource limits with `fpas-unit` and bundle decoding. PROGRAM-03 is a format-policy decision; document the chosen compatibility behavior in the program-image contributor documentation without describing unimplemented behavior first.
