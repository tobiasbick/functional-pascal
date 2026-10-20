# File handles and streaming

[`Std.Fs`](fs.md) opens regular files for incremental byte and UTF-8 line I/O.
Import `uses Std.Fs as Fs;`; cancellation-aware calls additionally take a token
from [`Std.Tasks`](../concurrency/task.md). Calls block their executing thread
and can run in `go` tasks with ordinary scope and `Tasks.Wait` ownership.

## Handles, aliases, and cleanup

`Fs.File` is an opaque handle owned by the VM that opened it. Copies refer to
the same open file, logical position, and close state. Reading through one
alias advances the position seen through the others. Calls sharing a handle
are serialized; independent concurrent readers still compete for that shared
position.

`Fs.Open(Path, Mode)` returns `Result of (Fs.File, Fs.FsError)`. Only regular
files are supported. Parent directories must already exist. Open validates an
existing entry as regular before truncating it in Create mode.

| `Fs.FileMode` | Behavior |
|---------------|----------|
| `Read` | opens an existing file for reading |
| `Create` | creates a writable file or truncates an existing file |
| `Append` | creates a writable file if needed; writes append to its end |
| `ReadWrite` | opens an existing file for reading and writing |

`Fs.Close(File)` returns `Ok(true)` and invalidates **every alias**. Further
operations, including another Close, return `Fs.FsError.Closed(Operation)`.
Explicitly close handles when finished; VM cleanup releases files still open
when that VM ends. A lexical task scope joins tasks but is not a file-close
scope.

## Byte reads and writes

`Fs.Read(File, MaxBytes)` returns `Result of (array of integer, Fs.FsError)`.
`MaxBytes` must be in `1..1048576`. The result has up to that many byte values
in `0..255`. An empty array means EOF. A nonempty short read is valid; callers
can continue reading until the empty result.

`Fs.Write(File, Bytes)` returns `Result of (integer, Fs.FsError)`. At most
1_048_576 bytes are accepted. The entire size and byte-domain validation runs
before any file I/O: elements outside `0..255` return
`InvalidByte(Offset, Value)` with the first invalid zero-based element index.
Invalid input does not advance the handle's logical position or discard its
unread line buffer.

The success value is the number of bytes actually written. It may be smaller
than the input length; write the remaining slice if the application needs a
complete write. An empty array returns a zero count. These are ordinary file
writes, without atomic publication or rollback after host errors.

## UTF-8 lines

`Fs.ReadLine(File)` returns `Result of (Option of string, Fs.FsError)`:

- `Some(Text)` is the next line, excluding its LF or CRLF terminator.
- `Some('')` is an actual blank line.
- `None` is EOF with no remaining content.
- An unterminated final line is returned once as `Some(Text)`.

A bare CR that is not immediately before LF remains content. NUL is valid
UTF-8 content. Each line must be valid UTF-8; an invalid line returns the
existing `FsError.InvalidEncoding(Path, Detail)` after consuming that entire
line, including its terminator. A later call reads the following line.

Content is limited to **1_048_576 bytes**, excluding LF/CRLF. An overlong line
returns `LimitExceeded(Operation, 1048576)` without draining an arbitrarily
long line. It retains a bounded prefix of at most 1 MiB + 2 bytes, leaving
the logical position unchanged. Repeating ReadLine repeats the limit error
until the caller recovers with Read or Seek.

```pascal id=example-lines run
program FileLines;

uses Std.Fs as Fs, Std.Test as Test;

begin
  const FileName := Test.ScratchDir() + '/file-lines-example.txt';
  Test.AssertTrue(Fs.WriteBytes(FileName, [104, 105, 13, 10, 10]).Unwrap());
  const File := Fs.Open(FileName, Fs.FileMode.Read).Unwrap();
  Test.AssertTrue(Fs.ReadLine(File).Unwrap() = Some('hi'));
  Test.AssertEquals(4, Fs.Tell(File).Unwrap());
  Test.AssertTrue(Fs.ReadLine(File).Unwrap() = Some(''));
  Test.AssertTrue(Fs.ReadLine(File).Unwrap() is None);
  Test.AssertTrue(Fs.Close(File).Unwrap());
  Test.AssertTrue(Fs.DeleteFile(FileName).Unwrap());
end.
```

## Positions, mixing operations, and Flush

`Fs.Tell(File)` returns `Result of (integer, Fs.FsError)` with the current
logical **byte** offset. Buffered lookahead does not make Tell report a later
position than the bytes consumed by the caller.

`Fs.Seek(File, Offset, Origin)` returns the resulting byte offset. Origin is
`Fs.SeekOrigin.Start`, `Current`, or `FileEnd`; Offset is signed. Invalid positions
return an Error. Seek accounts for buffered input, and subsequent reads use
the new position. Append-mode writes still append even after a seek.

Read drains unread buffered bytes first. Write starts at the logical position,
accounting for line lookahead; switching from ReadLine to byte I/O does not
silently skip prefetched content. All aliases observe the resulting position.

`Fs.Flush(File)` returns `Result of (boolean, Fs.FsError)`. It flushes output
and returns `Ok(true)` on success. It does not promise an operating-system
`fsync` or durable storage after a crash.

## Cancellation

The cancellation-aware forms have the same results as their ordinary forms:

| Call | Result success type |
|------|---------------------|
| `ReadWithCancellation(File, MaxBytes, Token)` | `array of integer` |
| `WriteWithCancellation(File, Bytes, Token)` | `integer` |
| `ReadLineWithCancellation(File, Token)` | `Option of string` |

An already cancelled token returns `FsError.Cancelled(Operation)` before
transferring file data. Active operations check cancellation between chunks;
an in-progress blocking host call is not guaranteed to be interrupted.
Cancellation does not undo completed I/O.

If ReadLine is cancelled after reading part of a line, that prefix remains
buffered and logically unconsumed. A later ReadLine resumes the same line;
Read can consume the prefix instead, and Tell, Seek, and Write account for it.

## Limits and errors

| Limit | Value |
|-------|-------|
| Simultaneously open files per VM | 1024 |
| Read request / Write input | 1_048_576 bytes |
| UTF-8 line content excluding LF/CRLF | 1_048_576 bytes |

Limits use `FsError.LimitExceeded(Operation, Limit)`. The operation names the
actual public function, for example `Std.Fs.Open`, `Std.Fs.ReadLine`, or
`Std.Fs.ReadLineWithCancellation`. Closed and Cancelled use the same operation
labels. Host failures retain the existing typed FsError variants and original
[`HostError`](errors.md) causes. The whole-file helpers retain their separate
contracts: ReadText allows 64 MiB, while ReadBytes/WriteBytes allow 1 MiB.

## See also

- [Filesystem API](fs.md)
- [Cancellation sources and tokens](../concurrency/task.md#cooperative-cancellation)
- [Task scopes](../../language/concurrency/task-scopes.md)
