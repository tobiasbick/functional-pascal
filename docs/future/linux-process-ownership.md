# Linux process ownership isolation

`OwnedProcess` and `PseudoTerminal` enable Linux child adoption in their host
process. Cleanup waits for children in the owned process group. The adoption
setting also applies to other process groups and stays enabled after disposal.

On Linux, start and dispose an `OwnedProcess` running `/bin/true`. Start a
separate shell through `System.Diagnostics.Process` and let the shell exit
while its child is still running. The host adopts the unrelated child. When
the child exits, it remains a zombie until it is explicitly reaped or the host
exits.

- [ ] Isolate child adoption in a supervisor that owns only the launched
  process tree. Preserve the root PID and exit status, startup suspension,
  terminal I/O, and group termination when the original parent exits first.
- [ ] Verify that disposing either API leaves unrelated child ownership
  untouched, including while other process or terminal sessions are active.

The affected paths are [OwnedProcess.cs](../../src/Fpas.Processes/OwnedProcess.cs)
and [LinuxTerminal.cs](../../src/Fpas.Processes/Terminal/LinuxTerminal.cs).
