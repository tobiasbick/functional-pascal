# Bytecode and portability contract

## Compatibility policy

`.fpascp` is portable between operating systems and CPU architectures, but remains an internal derived
format rather than a long-term package boundary. Portability means a compatible decoder/VM can run the
same bytes on another supported host. It does not mean arbitrary future FPAS releases must accept old
bytecode.

The register cutover increments both:

- `PROGRAM_FORMAT_VERSION` for the envelope/section schema;
- `BYTECODE_VERSION` for opcode and execution semantics.

An incompatible direct `.fpascp` run fails with a message that includes the image version, runtime
version, and the exact command concept needed to rebuild from sources. Project-backed commands rebuild
derived artifacts automatically.

## Forbidden persistent data

No `.fpascp` or `.fpascu` field may persist:

- `usize`, `isize`, native pointers, references, vtables, `TypeId`, or Rust enum memory layout;
- native-endian integer bytes;
- `PathBuf` platform representation;
- absolute source, workspace, toolchain, user-home, or temporary paths;
- hostnames, usernames, CPU model strings, or build-machine metadata;
- target triples or an assumption that pointer width is 64 bits;
- hash-map iteration order;
- executable machine code or relocation addresses from the host linker.

Persist only fixed-width integers, explicit byte strings/UTF-8, exact IEEE-754 bit patterns, digests,
and bounded ordered collections.

## Envelope

Retain the existing eight-byte magic. The recommended version-two envelope is:

```text
magic[8]
program_format_version: u16 LE
bytecode_version: u32 LE
flags: u16 LE (must be zero until assigned)
compiler_version_len: u32 LE
compiler_version: UTF-8 bytes
source_hash[32]
options_hash[32]
linked_unit_count: u32 LE
linked_unit identities...
payload_len: u32 LE
payload_digest[32]
payload bytes
```

Keep identity outside the executable payload so build reuse can inspect it without decoding all code.
All counts are checked against configured maxima before allocation or multiplication.

The payload digest is calculated over the exact payload bytes and verified before section decoding.

## Sectioned payload

Use a deterministic section directory followed by section bytes. Required sections:

| Tag | Contents |
|---|---|
| 1 | UTF-8 string table |
| 2 | persistent constants |
| 3 | source path table |
| 4 | globals |
| 5 | record layouts |
| 6 | enum layouts |
| 7 | function metadata |
| 8 | packed instructions |
| 9 | sparse source map |
| 10 | executable entry/feature metadata |

Each directory entry contains fixed-width tag, offset, length, and item count fields. Rules:

- tags are strictly increasing and unique;
- offsets are relative to payload start and use checked arithmetic;
- sections do not overlap, exceed the payload, or leave unexplained bytes;
- all required tags appear exactly once;
- unknown required tags are errors; future optional tags require a separately specified flag policy;
- item counts are checked before reserving vectors;
- decoder limits are at least as strict as encoder limits.

Do not use Serde for the core register executable. Write explicit encoders/decoders so integer width,
endianness, unknown values, resource accounting, and compatibility are reviewable. Serde may remain for
unrelated user formats.

## Instruction encoding

Write every `Instruction(u64)` as `word.to_le_bytes()` and read with `u64::from_le_bytes`. Decode and
validate the opcode and form after conversion. Do not mmap or cast the input payload to an instruction
slice; ARM alignment and host endianness must be irrelevant.

The persistent encoding is the packed logical word, not a promise about Rust's struct layout.

## Constants

Supported persistent constants initially match current runtime-independent constants:

- integer: signed `i64` two's-complement bytes;
- real: exact `f64::to_bits()` as `u64`;
- boolean: canonical byte `0` or `1`; other values are errors;
- string: `StringId` into the validated UTF-8 table;
- Unit: tag only;
- non-capturing function: `FunctionId` plus task-bound flag when semantically valid.

Do not persist arrays, dictionaries, records, enums, cells, tasks, captured closures, locks, or host
resources as constants unless a separately tested design extends the contract.

Constant deduplication uses semantic bit identity. In particular, preserve distinct NaN payloads and
signed zero as currently required by deterministic constant identity.

## Resource limits

Define limits in one module used by encoder, decoder, builder, and verifier. At minimum limit:

- total payload bytes;
- sections and section directory bytes;
- total strings and cumulative UTF-8 bytes;
- source paths;
- constants, globals, functions, record layouts, enum layouts, fields, and variants;
- instructions globally and per function;
- source runs;
- registers per function and call arguments;
- linked units and identity strings.

Use checked addition and multiplication before slicing, allocation, or `Vec::with_capacity`. An
overflow is a structured format error, never a panic or wrapped count.

Keep current limits where the representation still matches. Any changed user-visible compiler/runtime
limit must be documented under `docs/pascal/` at implementation time.

## Executable verifier

The decoder constructs an untrusted candidate. The VM accepts only a verified `Executable`.

For every function verify:

- nonempty valid code range and no overlap with another function;
- register count is representable and covers parameters/captures;
- each instruction opcode is known and uses its declared form;
- every register operand is below `register_count` and not the sentinel;
- every constant/string/global/function/type/field/variant/intrinsic ID exists;
- every argument window fits and arithmetic on base/count cannot overflow;
- direct-call arity matches target metadata;
- branch targets remain within the current function and target an instruction boundary;
- the control-flow graph reaches only valid terminators and cannot fall off the code range;
- return convention matches function metadata;
- aggregate layout operands agree with their owning type;
- source runs are sorted, unique by start, in range, and refer to valid source paths;
- feature flags such as task spawning agree with emitted operations.

Validation errors name function ID/name, instruction address, opcode, operand, actual value, and valid
range. Messages must be useful to coding agents.

After verification, encode the invariant in a distinct type such as `VerifiedExecutable`. Do not expose
an unchecked public constructor that the VM can accidentally accept.

## Deterministic encoding tests

Required tests:

- encoding the same executable twice produces identical bytes;
- construction order differences in internal maps do not affect bytes;
- canonical name case differences produce identical linked identities where semantics are
  case-insensitive;
- a fixed canonical executable produces a committed expected digest on every target;
- exact round-trip of code, constants, tables, layouts, source runs, and identity;
- exact real-bit round-trip for normal values, infinities, signed zero, and multiple NaN payloads;
- every truncated prefix fails without panic;
- trailing bytes, overlapping sections, duplicate tags, missing tags, bad offsets, bad digests, invalid
  UTF-8, unknown opcodes, noncanonical booleans, invalid IDs, and excessive limits fail;
- fuzz-like deterministic mutation of valid bytes never panics or allocates beyond bounds.

Do not commit generated `.fpascp` build outputs. Store a small expected digest or byte literal in Rust
test source when a golden wire contract is needed.

## Cross-host procedure

For each available producer host:

1. Build the same checked-in fixture project with a release `fpas` from one source revision.
2. Record only artifact digest, FPAS version, program/bytecode version, OS family, and architecture in
   the test report; never record machine-identifying paths or hostnames.
3. Copy the `.fpascp` bytes unchanged to every available consumer host.
4. Run `fpas run fixture.fpascp -- <fixed args>` with a compatible release.
5. Compare exit code, stdout, stderr, and any deterministic file outputs.
6. Run one fixture using relative Windows-style diagnostic paths and one using slash-style paths.

Minimum meaningful pairs when hosts are available:

- Windows x86-64 producer -> Linux x86-64 consumer;
- Linux x86-64 producer -> Windows x86-64 consumer;
- x86-64 producer -> ARM64 Linux or macOS consumer;
- ARM64 producer -> x86-64 consumer;
- one producer -> FreeBSD consumer.

A cross-compile-only `cargo check --target` is useful but does not satisfy artifact execution. Claim a
platform only after native execution or clearly label it compile-only/unverified.

## Host-native bundles

`fpas build --executable` remains host-native. It packages a host runner plus portable `.fpascp`.
A Windows `.exe` is not a Linux executable, even though the embedded program image is portable.

Do not add native cross-compilation to this rewrite. Keep bundle format validation and atomic
publication behavior intact.
