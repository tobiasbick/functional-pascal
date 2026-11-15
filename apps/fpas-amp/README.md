# FPas-Amp

FPas-Amp is a desktop music player in development, written in Functional Pascal.
Open **Settings** at the bottom of the sidebar. **General** controls the
appearance and accent color. **Library** lets you add a music folder, edit its
name and formats, or mark it offline. New sources start with FLAC, MP3 and M4A.
Appearance and offline changes save immediately. Use **Save** to confirm a new
or edited source.

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
  an absolute `path`, `offline` and an `extensions` array such as
  `[".flac", ".mp3"]`. Music sources include subdirectories.
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

A missing file yields an empty source list or a 1000 × 680 window with no saved
position. When a file is damaged, FPas-Amp saves the original bytes in a
`*.corrupt-*.bak` sibling and writes a repaired JSON file with the recoverable
values and defaults. A saved position on a monitor that is no longer connected
is left to the system.
