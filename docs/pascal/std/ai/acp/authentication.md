# Authentication

Initialize first, inspect AuthMethods and choose an advertised method. Agent
methods use the typed `authenticate` request. Authentication-required errors
remain peer errors; the application decides when to authenticate and retry.
See [sessions](sessions.md).

Terminal methods use `Host.RunTerminalAuthentication(Launch, Method,
TimeoutMillis, CancellationToken)`. The base command and working directory are
preserved, method args are appended once, and method environment values override
base entries using the host OS's key comparison. The base and merged launch
are validated through the common process implementation.

V2 uses `Host.V2RunTerminalAuthentication` with `V2.AuthMethodTerminal` and its
array of environment entries. It shares the same launch and foreground process
implementation; v1 and v2 method values are nominally distinct.

The operation requires an explicitly installed interactive process terminal.
Without one it returns UnsupportedCapability before launching anything. Local
cancellation and timeout return their typed errors; process-group cleanup uses
the existing Std.Os.Proc foreground implementation. Normal exit zero returns true;
nonzero exit or a signal returns false. ACP protocol pipes are never reused.

Both authentication variants use the owned ACP job path in pool tasks and ordinary
all-stop debugging. Their timeout starts before detached argument conversion and
includes native admission, the terminal provider, startup and execution. Cancellation
and timeout still join process-group cleanup. Host providers already in progress
must return; providers may request VM shutdown without joining jobs under their
own locks.

After true, close the old ACP connection, reconnect with the original base
launch and initialize again. No prompt or session is replayed automatically.
The application controls consent and any retry. Logout is a separate protocol
request guarded by the peer's logout capability.
