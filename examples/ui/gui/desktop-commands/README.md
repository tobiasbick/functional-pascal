# Native desktop commands

Build/run `DesktopCommands.fpasproj` with the local FPAS SDK. The example uses
CUI's native Open File, Save File and Choose Folder dialogs, including optional
messages when no selection is returned. **Save destination** returns a path to
the application. Linux needs Zenity or KDialog on `PATH`.

A confirmed selection updates the document title. The native close button sends
one typed MVU message. **Keep open** returns `Closing.Cancel`; **Close** returns
the shared `Mvu.Exit`. Repeated close attempts share the current decision.

**Request native repaint** requests a frame within half a second, or sooner for
input/animation. It neither changes the model nor creates a timer task. Use
`Mvu.StartBackground` and typed messages for time-driven application updates.

Creation options set native minimum dimensions and Linux desktop identity.
See the [GUI reference](../../../../docs/ui/gui/host.md) for size, placement,
fixed-size windows, owner-thread rules and cleanup.

## Appearance and layout

Choose Light, Dark or System, select an accent and drag the divider between
the panes. **Save settings** writes `fpas-desktop-commands/settings.json` under
.NET's per-user `LocalApplicationData` directory. The initial MVU model loads
the file before opening the window. A missing or unreadable file uses System,
Neutral and an even split. Missing or invalid fields use their own defaults.

```json
{
  "appearance": "dark",
  "accent": "violet",
  "splitRatio": 0.6
}
```

The save runs in an MVU background command. It writes a complete temporary
file before replacing `settings.json`. A typed result updates the feedback
below the settings controls. If a write fails, the current choices stay in
the view and **Save settings** becomes available for another attempt.

## Background progress

**Start** runs a simulated job that updates the progress bar on eight timer
ticks. The worker uses [Mvu.StartBackground](../../../../docs/ui/mvu/runtime.md#background-effects),
waits with the cancellation token and posts a typed `WorkTick` after each
delay. The updates wake the GUI even when there is no user input.

**Cancel** keeps the last displayed value and invalidates any pending replies
from that run. The next start resets progress to zero. **Simulate error** runs
the same job and fails after two successful steps. The failure mapper posts
`WorkFailed` so the view can show the error and offer another start.

## Source files

- `examples/ui/gui/desktop-commands/`
  - [DesktopCommands.fpasproj](DesktopCommands.fpasproj)
  - [Background.fpas](Background.fpas)
  - [Preferences.fpas](Preferences.fpas)
  - [State.fpas](State.fpas)
  - [Update.fpas](Update.fpas)
  - [View.fpas](View.fpas)
  - [main.fpas](main.fpas)
- [DesktopExampleContract.fpas](../../../../tests/Fixtures/gui-host/DesktopExampleContract.fpas)
