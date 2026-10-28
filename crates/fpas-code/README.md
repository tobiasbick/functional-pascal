# Portable code

`fpas-code` is the shared contract between compilation, persistent program
artifacts, and execution. It owns instructions and intrinsic identifiers,
portable constants and metadata, debug information, and executable validation.
It does not depend on runtime values, memory domains, or host services.

`Executable::verify` returns an immutable `VerifiedExecutable` with decoded
instructions and canonical function names. Its opaque identity binds
`FunctionTarget` values to one verified image. Sharing the image through `Arc`
preserves identity; independent verification or cloning creates a fresh one.

The runtime prepares value-specific caches, such as shared string constants,
when admitting code. Program encoding remains in `fpas-program`.

See the [workspace architecture](../../docs/rust-architecture.md).
