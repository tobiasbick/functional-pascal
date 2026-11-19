# Native desktop commands

Build/run `DesktopCommands.fpasproj` with the local FPAS SDK. The example uses
CUI's native Open File, Save File and Choose Folder dialogs, including optional
messages when no selection is returned. Save selects a destination; it does
not write a file. Linux needs Zenity or KDialog on `PATH`.

A confirmed selection updates the document title. The native close button sends
one typed MVU message. **Keep open** returns `Closing.Cancel`; **Close** returns
the shared `Mvu.Exit`. Repeated close attempts share the current decision.

**Request native repaint** requests a frame within half a second, or sooner for
input/animation. It neither changes the model nor creates a timer task. Use
`Mvu.StartBackground` and typed messages for time-driven application updates.

Creation options set native minimum dimensions and Linux desktop identity.
See the [GUI reference](../../../../docs/ui/gui/host.md) for size, placement,
fixed-size windows, owner-thread rules and cleanup.
