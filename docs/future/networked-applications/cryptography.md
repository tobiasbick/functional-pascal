# Future: Cryptography

> Deferred. No general cryptographic unit is currently implemented.

Secure networked applications need primitives whose safety does not depend on application code
assembling low-level algorithms correctly. The interface should prefer complete operations with
safe defaults over a large collection of interchangeable primitives.

## Proposed scope

- Operating-system cryptographic random bytes and uniformly sampled secure integers.
- Password hashing and verification through Argon2id with encoded parameters and an upgrade check.
- SHA-256 and SHA-512 digests for interoperability and integrity use cases.
- HMAC-SHA-256 for authenticated application tokens and messages.
- Ed25519 key generation, signing, and verification for persistent node identity.
- Constant-time comparison for authentication tags and other fixed-length secret values.
- Bounded Base64 and hexadecimal codecs if the existing text/byte conventions cannot express the
  required wire formats cleanly.

## Interface rules

- Secure defaults must be selected by the module, while stored password hashes retain their exact
  cost parameters for later upgrades.
- Randomness failure must be reported; the implementation must never fall back to `Std.Random`.
- Verification returns an ordinary false result for a valid but non-matching value and a distinct
  error for malformed encodings or unavailable facilities.
- Private keys and raw secret material must not implement accidental diagnostic rendering.
- Algorithms considered obsolete for new security work must not be added for convenience.

## Open decisions

- Whether secret byte storage needs a dedicated non-copyable value before private-key operations
  can be exposed safely.
- Whether TLS certificate parsing belongs here or remains entirely inside `Std.Net`.
- Which reviewed Rust crates supply each operation and how dependency auditing is maintained.

## Acceptance requirements

- Published test vectors pass for every deterministic primitive.
- Password tests cover correct, incorrect, malformed, and parameter-upgrade cases.
- Signatures fail for changed payloads, wrong keys, malformed encodings, and non-canonical inputs.
- Random generation uses the operating system on every supported target and surfaces failure.
- Secret values do not appear in errors, logs, snapshots, or test output.
