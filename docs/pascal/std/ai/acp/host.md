# `Std.Acp.Host`

Aliases for stable v1 filesystem and terminal request/response records. These
aliases retain the exact nominal identity of their `Std.Acp.Types` declarations.

The pairs are `ReadTextFile`, `WriteTextFile`, `CreateTerminal`, `TerminalOutput`,
`WaitForTerminalExit`, `KillTerminal`, and `ReleaseTerminal`, each with `Request`
and `Response` suffixes. Import `uses Std.Acp.Host as Host;` for the payloads.
Host execution helpers are a later AP50 package; constructing these records
performs no I/O.

See [protocol values and codecs](types.md).
