# `Std.Acp.Host`

Explicit filesystem, terminal and interaction adapters. Import `Std.Acp.Host`
and retain the original typed request and Responder from the connection event.
Receiving a request grants no host authority. Applications may refuse it,
return an unsaved editor buffer, or invoke a helper after their own policy check.
All request/response aliases retain their `Std.Acp.Types` nominal identity.

## Execution and response rights

`ReadTextFile`, `WriteTextFile`, `CreateTerminal`, `TerminalOutput`,
`WaitForTerminalExit`, `KillTerminal` and `ReleaseTerminal` take the original
Responder and the matching typed request. They return the corresponding typed
response in `Result of (..., Types.AcpError)`. Wait also takes
`TimeoutMillis: Option of integer` and `CancellationToken: Option of
Tasks.CancellationToken`. `None` disables the respective bound.

Helpers verify method, exact decoded arguments, connection/session and live
request identity before admitting effects. A separate atomic execution right
allows one helper call per Responder. Invalid arguments and resource admission
failures leave that right available. After admission, success and failure both
finish it: a competing running call gets Busy; another completed call gets
InvalidArgument. A reply while the helper is running gets Busy. The application
still owns the separate response right and must call Respond or RespondError.
Cancellation before admission prevents the effect; completed I/O is not undone.

For example, an application passes its policy decision explicitly:

```pascal id=acp-approved-file-dispatch
program ApprovedFileDispatch;
uses Std.Acp as Acp, Std.Acp.Host as Host, Std.Acp.Types as Types;

function AnswerRead(Responder: Acp.Responder; Request: Types.ReadTextFileRequest; PermitDisk: boolean): Result of (boolean, Types.AcpError);
begin
  if not PermitDisk then
    return Acp.RespondError(Responder, Types.ProtocolError(Code := -32000, Message := 'Disk access denied', Data := None));
  end if;
  case Host.ReadTextFile(Responder, Request) of
    when Ok(const Reply):
      return Acp.Respond(Responder, Types.ClientResponse.ReadTextFileResponse(Reply));
    when Error(const Cause):
      if Cause is Types.AcpError.Closed(_) or Cause is Types.AcpError.Cancelled(_) then
        return Error(Cause);
      end if;
      return Acp.RespondError(Responder, Types.ProtocolError(Code := -32603, Message := 'Host read failed', Data := None));
  end case;
end function;
begin
end.
```

## Files

Paths must be absolute and contain no NUL. This adapter serves one explicit
path; it is not a workspace sandbox. A root policy belongs to the application.
Reads use the existing bounded Std.Fs UTF-8 reader. Line numbers start at one;
an omitted line starts at the beginning, an omitted limit reads the remainder,
zero limit returns empty text, and a range after EOF is empty. CRLF/LF and a
missing final terminator are preserved. Invalid UTF-8 and I/O failures return
Host errors. Response construction is bounded by MessageBytes.

Writes use the same implementation as Std.Fs.WriteText. They create a missing
file in an existing parent; they do not create missing parent directories.
Editor clients may instead construct ReadTextFileResponse from their own buffer
and call Respond without touching disk.

## Terminals

Create returns a terminal ID immediately after launch. Command, args, cwd and
environment stay separate. The existing process registry owns the process
group and pipes. Stdout/stderr are continuously drained on the ACP executor;
their combined output uses arrival-observed order across the two OS pipes.
UTF-8 is decoded incrementally with replacement for invalid/incomplete bytes.
The retained suffix is bounded by UTF-8 bytes and cuts only at scalar boundaries.

Terminal count and output capacity are reserved before launch. OutputByteLimit
defaults to Limits.TerminalBytes and cannot exceed it; the aggregate reservation
cannot exceed AllTerminalBytes. Exited and killed terminals retain reservations
until release. Output and exact exit code or Unix signal remain queryable.
WaitForTerminalExit suspends its caller while dispatch and other tasks continue;
local cancellation/timeout does not kill the process or consume the response right.
An admitted wait still uses that Responder's execution right.

Terminal IDs belong to one connection and session. Kill retains the terminal;
release terminates/reaps remaining work and invalidates aliases. Repeated
release returns Closed. Successful session close releases its terminals and those
of its observed descendants, and invalidates unanswered host requests;
connection close and VM teardown release all remaining terminal resources.

## Permission and elicitation

Respond validates selected permission IDs against the original options. No
selection becomes a permanent host capability grant. Session cancellation
settles unanswered permissions as cancelled; stale responders cannot execute
helpers or publish a second answer.

`ElicitationDefaults(Responder)` returns editable form defaults as RawJson.
`ValidateElicitation(Responder, Response)` checks a submission without sending
it; Respond performs the same check before consuming the reply right. Validation
covers required fields, primitive types, ranges, lengths, enum/multiselect choices
and bounded pattern matching. Formats, titles and descriptions remain available
for UI presentation. Defaults never silently accept a form. Pattern syntax or
execution limits produce a typed error rather than an unchecked submission.

URL requests expose destination, elicitation ID and requesting scope in their
typed payload. Display the peer identity from State together with that scope,
obtain consent, and correlate completion in the same connection/UI interaction.
The library opens no browser and collects no credentials. See
[authentication](authentication.md) for separate terminal login.
