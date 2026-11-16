# FPas-Amp

FPas-Amp is a desktop music player in development, written in Functional Pascal.
Open **Settings** at the bottom of the sidebar. **General** controls the
appearance and accent color. Its **Library database** card clears all indexed
entries after confirmation and pauses every source. **Library** lets you add a
music folder, edit its name/formats, mark it offline or pause its scan.
New sources start with FLAC, MP3 and M4A.
Appearance, offline and pause changes save immediately. Use **Save** to confirm a new
or edited source.
The next launch returns to the settings tab that was open when the window closed.

Online music locations are discovered and monitored in the background by
[FpasAmp.Library](src/FpasAmp.Library/README.md). Discovery includes subdirectories
and the configured extensions, batches changes and retries unavailable locations.
It only records which entries exist; it does not read audio or tags. The inventory
shows each source's status and real file/directory counters. Running scans use an
indeterminate activity bar; no percentage is invented before the total is known.
Its reconstructible SQLite index is loaded before background reconciliation.

Each source has **Clear index**, **Rescan** and **Edit** actions. Clear index
removes only that source's associations after confirmation and automatically
enables **Pause**. Shared entries belonging to other sources and the actual music
files remain. Turning Pause off starts a complete reconciliation, including
changes made while paused. Rescan checks only the selected online, unpaused source.
It is disabled while that source is paused, offline or scanning.

Set up the [local FPAS SDK](../../docs/build/development.md#build-an-fpas-application)
and the [native CUI build tools](../../src/Fpas.Ui.Gui.Cui/native/README.md#prerequisites).
Run these commands from this directory:

```sh
dotnet build fpas-amp.slnx
dotnet run --project src/FpasAmp/FpasAmp.fpasproj --no-build
```

## User files

FPas-Amp reads two unversioned JSON files from the `fpas-amp` subdirectory of
.NET's `ApplicationData` folder. The default paths are `%APPDATA%\fpas-amp`
on Windows and `~/.config/fpas-amp` on Linux. Linux honors `XDG_CONFIG_HOME`.

- `settings.json` holds a `musicSources` array. Each entry stores `id`, `name`,
  an absolute `path`, `offline`, `paused` and an `extensions` array such as
  `[".flac", ".mp3"]`. Music sources include subdirectories.
  A missing `paused` value defaults to false. Pause and Offline are independent.
- `settings.json` also stores `appearance` (`light`, `dark` or `system`) and
  `accent` (`neutral`, `violet`, `blue`, `green`, `amber`, `orange`, `coral` or
  `rose`). Missing appearance fields default to system and neutral. System mode
  follows Windows app appearance while running and keeps the chosen accent.
  On X11 it uses CUI's dark `GTK_THEME` hint when available and otherwise
  uses light.
- `state.json` holds `width`, `height`, `x`, `y` and `maximized`. The
  dimensions describe the normal window in logical units. `x` and `y` are the
  top-left corner of its content in desktop coordinates: pixels on Windows
  and X11. They are `null` where the system reports no position, as on
  Wayland. FPas-Amp saves the file when its window closes and reopens the
  window there, maximized if it was. A window closed while minimized keeps
  the state it would have come back in.
- `state.json` also stores `settingsTab`, with `0` for General, `1` for Library
  and `null` for the empty shell. A missing field defaults to the empty shell.
  The selected page is saved when the window closes.

A missing file yields an empty source list or a 1000 × 680 window with no saved
position. When a file is damaged, FPas-Amp saves the original bytes in a
`*.corrupt-*.bak` sibling and writes a repaired JSON file with the recoverable
values and defaults. A saved position on a monitor that is no longer connected
is left to the system.

`library.db` stores discovered paths, source memberships and directory check states
under .NET's `LocalApplicationData/fpas-amp`, independently of the music locations.
The index survives restarts and temporary source outages. Only completed directory
checks remove absent entries. SQLite may also create `library.db-wal` and
`library.db-shm` sidecars. Database errors are reported without replacing the file.
