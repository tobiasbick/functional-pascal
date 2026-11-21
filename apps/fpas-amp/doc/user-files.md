# User files and recovery

## Settings and window state

FPas-Amp stores two unversioned JSON files in the `fpas-amp` subdirectory
of .NET's `ApplicationData` folder. On Windows this is
`%APPDATA%\fpas-amp`. On Linux it defaults to `~/.config/fpas-amp` and
honors `XDG_CONFIG_HOME`.

| File | Contents |
| --- | --- |
| `settings.json` | Music sources, appearance, accent, mixer settings, AcoustID keys and radio favorites. |
| `state.json` | Window size and position, maximized state, selected page, last Settings tab and local playback bookmark. |

Each `musicSources` entry stores an ID, name, absolute path, enabled
extensions and the `offline` and `paused` flags. `scanPlaylists` is
independent of audio file formats and defaults to true. `appearance`
defaults to `system` and `accent` defaults to `neutral`. The `mixer`
stores the volume and ten equalizer gains in dB. `radioFavorites` stores
the saved stations in sidebar order. AcoustID keys are in
`acoustId.applicationKey` and `acoustId.userApiKey` as plain text.

`state.json` records the normal window size (`width`, `height`) in logical
units and its content origin (`x`, `y`) in desktop coordinates. The origin
is `null` when the window system does not report it. `maximized` records
the maximize state, including the state a minimized window would restore to.
Window placement for a disconnected monitor is handled by the window system.

`settingsTab` is `0` for General, `1` for Library or `2` for Metadata.
`activePage` saves the selected destination, including an album, artist,
genre, playlist or station by stable ID. If an indexed collection disappears,
the app falls back to its parent or to Music. If a saved radio favorite
disappears, the app opens Stations.

The local playback bookmark stores the source and applied search, filters
and sort, plus the selected track's durable ID, slot, position and duration.
The app uses the library to recreate the queue and restores it paused.
See [playback after restart](playback.md#after-a-restart).

If either file is missing, the app starts with defaults, including an empty
source list and a 1000 × 680 window. If a file is damaged, the app saves the
original bytes in a `*.corrupt-*.bak` sibling, recovers usable fields and
writes repaired JSON with defaults for the rest.

## Library databases

The SQLite files are kept under .NET's `LocalApplicationData/fpas-amp`,
separately from the music folders.

- **`library.db`** holds the reconstructible index, directory state, tags,
  source playlists and analysis caches. It survives restarts and temporary
  source outages. Only completed directory checks remove missing entries.
  SQLite may create `library.db-wal` and `library.db-shm` sidecars.
- **`userdata.db`** holds durable track identities, ratings, local playlists
  and library-only tag overrides. Clearing the index preserves these data.
  A moved file can recover its identity when a unique full-file hash matches
  a previous path confirmed absent. Copies have separate identities.
- **`covers/thumbnails-v1`** holds cached cover thumbnails beside the
  databases. Cached covers can be shown while a source is offline.

See the [library persistence reference](library.md#reconciliation-and-persistence)
and the [artwork cache reference](imaging.md).
