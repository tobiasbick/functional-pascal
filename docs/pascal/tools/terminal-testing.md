# Controlled terminal tests

`fpas terminal test SCRIPT.json` runs an explicit executable in a private native
terminal: ConPTY on Windows, a POSIX terminal/session on Linux. It creates no
visible window and does not require an editor or TUI library.

## Script contract

```json
{
  "schemaVersion": 1,
  "executable": "/absolute/path/to/dotnet",
  "arguments": ["/absolute/path/to/Program.dll"],
  "columns": 80,
  "rows": 24,
  "timeoutSeconds": 30,
  "outputBytes": 1048576,
  "steps": [
    {"action": "expect", "text": "ready"},
    {"action": "resize", "columns": 100, "rows": 40},
    {"action": "send", "text": "hello\r"},
    {"action": "expect", "text": "received:hello"},
    {"action": "exit", "exitCode": 0}
  ]
}
```

Replace the executable and argument paths with the selected platform's actual
host and built application. This example assumes that application prints
`ready`, reads a line, prints `received:hello` and exits zero. It is a protocol
for a cooperative test target, not an assertion about arbitrary applications.
The [acceptance targets](../../../tests/Fpas.IntegrationTests/Cli/TerminalWorkflowTests.cs)
exercise actual FPAS Console APIs with non-redirected input, Unicode keys,
window size, natural exit and a waiting parent/descendant pair.

```sh
fpas terminal test .temp-data/terminal.json
fpas schema terminal-script
```

Input is strict UTF-8 JSON, at most 64 KiB. Required fields are executable,
arguments and steps. Arguments are literal tokens; no shell is involved. The
executable must be an absolute existing executable. Optional `directory` is the
target's working directory; its default is the script's parent. Other defaults
are shown above. Unknown members and action names are errors.

There must be 1..256 ordered steps, with `exit` last. Dimensions accept 1..500
columns and 1..300 rows. Timeouts accept 1..3600 seconds and output bounds accept
1..1048576 UTF-8 bytes.

| Action | Meaning |
| --- | --- |
| `send` | Write nonempty text as UTF-8 input and flush. Include the desired newline or control character explicitly. |
| `expect` | Wait for nonempty literal text after the previous successful expectation; advance that transcript cursor. |
| `resize` | Apply explicit columns and rows to the native terminal. |
| `exit` | Wait for natural process exit and require the supplied integer exitCode. |

Each action accepts only its documented fields. Expectations are case-sensitive
ordinal substring checks, not regexes, screen coordinates or cell assertions.
The transcript retains terminal sequences and echoes. Input and output share
the terminal's native modes; raw/cooked input belongs to the application. For
example `\u0003` is an input control character whose effect depends on that mode.

The CLI does not emulate a renderer or promise deterministic replay. A passing
transcript does not prove focus, layout or rendered Unicode cell geometry;
those belong to the TUI library's separate tests.

## Ownership and results

The result is one [operation JSON report](report-schemas.md), including PID,
natural exit when observed, completed-step count, transcript and truncation.
Output and operation deadline apply across the whole session. A failed
expectation, wrong exit code, early terminal closure, output limit, timeout or
cleanup failure is an explicit failed report. Argument/schema errors exit 2;
successful scripts exit 0, operational failures exit 1, cancellation exits 130.

All exits stop and join owned descendants. The `exit` expectation checks natural
completion before harness cleanup; a timeout cannot satisfy it. Windows owns a
job and closes the private pseudo-console; Linux owns a controlling terminal and
process group, adopts and reaps terminated descendants. Output is drained during
cleanup, including after a successful natural exit. Cleanup has its own bounded
grace period.

Supported native hosts are Windows x64 with ConPTY (Windows 10 1809+) and Linux
x64/glibc, including WSL. Native session/spawn APIs are required; redirected
managed streams alone cannot establish terminal behavior. Missing native support
fails the command. Each test owns an independent session and transcript.

## See also

- [Native framework tests](testing.md)
- [CLI report schemas](report-schemas.md)
- [Processes and cancellation](../libraries/processes.md)
