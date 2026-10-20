# Std.Errors

Shared native causes preserve the attempted hosted operation and its original explanation.

`HostError.Failure(Operation: string; Resources: array of string; Domain: Option of string; Code: Option of integer; Detail: string)` is a native error with the fixed label `Host operation failed`. Resources retain supplied identifiers in parameter order. The builder does not resolve paths or inspect resources.

Domain and Code are both present for a raw OS code (`errno` on Unix, `win32` on Windows), and both absent when the backend has no raw OS code. Detail is diagnostic text; branch on the surrounding domain variant rather than interpreting this text. Use `Message()` only at an output or explicitly text-returning application boundary.

Fs, Proc and Crypto retain this leaf in their Cause payload. Their error families remain distinct nominal types.
