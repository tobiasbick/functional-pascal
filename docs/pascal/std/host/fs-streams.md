# File handles and streaming

[`Std.Io.Fs`](fs.md) opens regular files for incremental byte and UTF-8 line I/O.
Import `uses Std.Io.Fs as Fs;`; cancellation-aware calls additionally take a token
from [`Std.Tasks`](../concurrency/task.md). Pool tasks and the all-stop debugger
suspend while handle operations run, including calls inside inline callbacks.

## Execution and native ownership

`Open`, `Close`, byte and line reads/writes, `Seek`, `Tell`, `Flush`, `Size`,
`SetLength` and the text-output functions save their task state while a native
file worker executes. No FPAS values or callback frames move to that native
thread. Results are converted to FPAS values on the owning VM task. Synchronous
main-program calls and native callback boundaries use the same file operations
on their calling thread.

Each VM admits at most 64 native file jobs, including completed results not yet
delivered to their task. Saturation returns `FsError.LimitExceeded(Operation, 64)`
before starting native work; it cannot truncate a file in Create mode. This job
budget is shared with whole-file/path operations and separate from the 1024 open-file
limit. Pending pool/debugger calls use the shared one-millisecond readiness polling
schedule.

Aliases serialize through the same handle state. Waiting for registry or handle
locks observes cancellation. A VM shutdown first signals jobs without waiting
for those locks; final descriptor cleanup follows task and native-worker joins.
Closing a handle retains its admission slot until its OS descriptor is released.

Abandoning a task signals and joins its native job, drops undelivered byte/text
results and closes a newly opened file that was not delivered to FPAS. Existing
handles remain independently VM-owned until explicit Close or VM teardown.
Already executed reads, writes, seeks, truncations and closes are not rolled back.
Portable filesystem calls cannot be forcibly interrupted: cancellation and cleanup
can wait for an OS call already in progress. Already dispatched native work may
also finish while all-stop debugging is paused.

Whole-file and path-based helpers also suspend pool/debugger tasks and use the same
native-job budget. Their plain queries wait for admission rather than changing their
return type. See [filesystem execution and cancellation](fs.md#execution-and-cancellation).

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

Independent handles and other processes are not synchronized by the VM's
per-handle serialization. Opens acquire no advisory file lock or exclusive
ownership. Windows uses Rust's default read/write/delete sharing; POSIX uses
ordinary open-file semantics. Host permissions and sharing rules still apply,
and other actors may change the file while a handle is open.

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

uses Std.Io.Fs as Fs, Std.Test as Test;

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

## Open-file size and length

`Fs.Size(File)` returns `Result of (integer, Fs.FsError)` with the open file's
current byte length. It works with every open mode, preserves the logical
position and buffered input, and queries the handle rather than reopening its
path. A separate `Fs.FileSize(Path)` queries a regular file by path. A size
observation does not prevent another handle or process from changing the file.

`Fs.SetLength(File, Length)` returns `Ok(true)` after resizing a writable file.
`Length` is a nonnegative byte count; negative values return
`FsError.InvalidArgument('Length', Detail)`. Read-only handles return
`FsError.InvalidArgument('File', Detail)`. Shrinking removes bytes; extending
adds bytes that read as zero. The 1 MiB transfer limit does not cap file length;
host filesystem and storage limits still apply.

The logical position is preserved, even beyond the new EOF. Buffered lookahead
and any retained line prefix are invalidated before resizing, so later reads
through aliases cannot return the old contents. A subsequent ordinary write
uses the preserved position; Append-mode writes use the current file end.
Resizing is serialized with operations through aliases. Host errors are returned
as typed failures without a rollback guarantee.

## Complete UTF-8 text output

`Fs.WriteString(File, Text)` and `Fs.WriteLine(File, Text)` return
`Result of (boolean, Fs.FsError)`. Success is `Ok(true)` after writing all the
UTF-8 bytes; WriteLine also appends exactly one LF byte on every platform.
No BOM is added and existing CR/LF characters in Text are not transformed.
NUL is ordinary content. An empty WriteString writes no bytes; an empty
WriteLine writes LF.

Each call accepts at most **1_048_576 UTF-8 content bytes**, excluding the LF
added by WriteLine. Validate the complete content size before I/O or discarding
buffered input. Text writes require a writable handle, use its logical byte
position, and retain Append-mode behavior. The entire operation, including LF,
is serialized against operations through aliases; independent handles and
external writers do not gain an atomic-line guarantee.

Host errors or cancellation may leave a prefix written, including incomplete
UTF-8 or a missing final LF. The error does not return a retry offset or roll
back output. Use byte writes when the application must manage partial progress.
Text writes do not flush or promise durable storage.

```pascal id=example-resize-text run
program ResizedText;

uses Std.Io.Fs as Fs, Std.Test as Test;

begin
  const Path := Test.ScratchDir() + '/resized-text.txt';
  const File := Fs.Open(Path, Fs.FileMode.Create).Unwrap();
  Test.AssertTrue(Fs.WriteLine(File, 'hello').Unwrap());
  Test.AssertEquals(6, Fs.Size(File).Unwrap());
  Test.AssertTrue(Fs.SetLength(File, 2).Unwrap());
  Test.AssertEquals(6, Fs.Tell(File).Unwrap());
  Test.AssertTrue(Fs.Close(File).Unwrap());
  Test.AssertEquals('he', Fs.ReadText(Path).Unwrap());
  Test.AssertTrue(Fs.DeleteFile(Path).Unwrap());
end.
```

## Cancellation

The cancellation-aware forms have the same results as their ordinary forms:

| Call | Result success type |
|------|---------------------|
| `ReadWithCancellation(File, MaxBytes, Token)` | `array of integer` |
| `WriteWithCancellation(File, Bytes, Token)` | `integer` |
| `ReadLineWithCancellation(File, Token)` | `Option of string` |
| `WriteStringWithCancellation(File, Text, Token)` | `boolean` |
| `WriteLineWithCancellation(File, Text, Token)` | `boolean` |

An already cancelled token returns `FsError.Cancelled(Operation)` before
transferring file data. Active operations check cancellation between chunks;
an in-progress blocking host call is not guaranteed to be interrupted.
Cancellation does not undo completed I/O.

Text output checks cancellation between bounded write chunks and before the
added LF. On failure the logical position reflects the bytes actually written.
Completing the final write is success even if cancellation is requested later.

If ReadLine is cancelled after reading part of a line, that prefix remains
buffered and logically unconsumed. A later ReadLine resumes the same line;
Read can consume the prefix instead, and Tell, Seek, and Write account for it.

## Limits and errors

| Limit | Value |
|-------|-------|
| Simultaneously open files per VM | 1024 |
| Pending or undelivered native handle jobs per VM | 64 |
| Read request / Write input | 1_048_576 bytes |
| UTF-8 line content excluding LF/CRLF | 1_048_576 bytes |
| UTF-8 write content excluding added LF | 1_048_576 bytes |

Limits use `FsError.LimitExceeded(Operation, Limit)`. The operation names the
actual public function, for example `Std.Io.Fs.Open`, `Std.Io.Fs.ReadLine`, or
`Std.Io.Fs.ReadLineWithCancellation`. Closed and Cancelled use the same operation
labels. Host failures retain the existing typed FsError variants and original
[`HostError`](errors.md) causes. The whole-file helpers retain their separate
contracts: ReadText allows 64 MiB, while ReadBytes/WriteBytes allow 1 MiB.

## See also

- [Filesystem API](fs.md)
- [Cancellation sources and tokens](../concurrency/task.md#cooperative-cancellation)
- [Task scopes](../../language/concurrency/task-scopes.md)
