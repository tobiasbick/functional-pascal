# FPas-Amp

FPas-Amp is a desktop music player in development, written in Functional Pascal.
The navigation groups **Music**, **Albums**, **Artists**, **Genres** and **Ratings**
under **Library**, followed by all discovered/local **Playlists** in name order.

**Music** lists all songs across your sources in a virtual table. Each row shows the
album cover, title, performer, album, genre, duration and track rating. Search across
titles, performers and albums, or filter by performer, album, genre and minimum track
rating. The album filter distinguishes releases with the same title. Click a column header or use the sort menu to change the order. The status
bar shows the matching count and playing time. Playing a row uses this selection and
order for the queue. The list follows library scans while open.

**Artists** groups releases by individual album artist. Collaborations appear under each
credited artist. Each tile shows the first available cover in catalogue order, album
and song counts, and ten stars for an independent artist rating. Open an artist to
browse their albums, then open an album for its tracks. Back returns one level at
a time and the sidebar keeps **Artists** selected. The artist grid filters by genre
and artist rating. A discography filters by genre and album rating. Genre choices
appear when they distinguish entries in the current collection.

**Genres** groups tracks by their genre tags. Each tile shows the first available
album cover, album and song counts, and an independent ten-star genre rating.
An album can appear in several genres. Opening it through **Genres** limits its
song count, track list and playback queue to the selected genre. A song with
multiple genres counts once in each. Songs without a genre remain accessible
through **Albums**. The genre overview filters by genre rating, its albums by
album artist and album rating, and its tracks by track rating.

**Albums** shows the full virtual cover grid. Its search field finds albums by title, artist
and genre as you type: words in any order, any of them, with typos, accents and
sound-alike spellings forgiven, best matches first. **Filters** narrows the grid
to one album artist, one genre or a minimum rating once applied, and its button
shows how many filters are active. The sort menu orders by title, artist, rating or song
count in either direction, and by best match while searching; clearing the search
returns to the order chosen before. The status bar shows how many albums match. The playlist list updates
in the background; same-name source lists show their location alongside the name.
Open an album to see its cover, title and album artist beside the track list. Each row
shows the performer below the track title, followed by duration, rating and a menu.
The open album updates as the catalogue changes. If the release disappears,
the view returns to its parent grid. The table uses
disc and track order by default. It uses the same toolbar as the album grid, with
title and genre search. The filter popup always offers a minimum rating. It also
lists genres that occur on some of the album's tracks and hides the genre field
when every track has the same set of genres. Rate a
track with its ten stars or use the row menu to inspect its tags, copy the path
from the details or open the folder.
The player strip stays at the bottom of every destination except **Settings**.
The play button beside each title starts that track and queues the shown tracks
after it. Until then, the strip shows **Nothing playing** with its transport and
position bar disabled; the equalizer and volume stay editable. It shows
the title and artist, previous/play-pause/next, a ten-band equalizer (±12 dB with
**Reset**), a vertical volume fader (−60 to 0 dB) beside the stereo level meters,
and a full-width waveform. **Previous** restarts a track after three seconds. Below 760 logical
pixels of content width, the strip hides the equalizer and meters. The meters show
the output after equalization and volume. Click the waveform to seek, or drag
to preview a position and release to seek once. Both work while playing or paused.
The accent color marks the played portion. On first playback, waveform analysis
starts in the background once playback has delivered audio. You can seek while it loads.
The library index caches the envelope by SHA-256 for later plays and app restarts.
The dropdown beside **Reset** offers Rock, Pop, Jazz, Classical, Bass Boost and Vocal.
**None** and **Reset** set all ten bands to zero. A manually adjusted curve is
shown as **Custom** when it differs from every preset.
At the end of a song the next queued track starts automatically. Playback
stops at the end of the queue. Queue navigation skips tracks that belong only
to offline sources, even when the file remains readable. Taking the current
source offline advances to the next eligible track. Removing or resetting its
last online owner stops playback, as does clearing the whole index. A track
shared with another online, indexed source remains playable. Pausing a scan
keeps indexed tracks available for playback.
The status bar shows the matching track count and duration. The back arrow restores
the album grid's search, applied filters, sort order and scroll position.
Below 760 logical pixels, the sidebar becomes a left-hand drawer opened through
the menu button. Selecting a destination closes it. **Settings** stays at the bottom.

Open **Settings** at the bottom of the sidebar. **General** controls the
appearance and accent color. Its **Library database** card clears all indexed
entries after confirmation and pauses every source. **Library** lets you add a
music folder, edit its name/formats, mark it offline or pause its scan.
New sources start with FLAC, MP3 and M4A.
Appearance, offline and pause changes save immediately. Use **Save** to confirm a new
or edited source.
The next launch restores the selected destination and independently remembers the
settings tab. Playlist selection uses its store/UUID, never its label or position.
It waits for a successful background snapshot before validation. A missing playlist
falls back to **Music**; a retained playlist with a parse failure opens with a warning.

Online music locations are discovered and monitored in the background by
[FpasAmp.Library](src/FpasAmp.Library/README.md). Discovery includes subdirectories
and the configured extensions, batches changes and retries unavailable locations.
Directory discovery records which entries exist. A second worker uses ATL to
read tags/audio properties and computes SHA-256 without decoding audio. The inventory
shows each source's status and real file/directory counters. Running scans use an
indeterminate activity bar; no percentage is invented before the total is known.
These counters describe discovery; tag processing follows independently.
Its reconstructible SQLite index is loaded before background reconciliation.

Each source has **Delete**, **Reset**, **Refresh** and **Edit** actions, in that order. Reset
removes only that source's associations after confirmation and automatically
enables **Pause**. Shared entries belonging to other sources and the actual music
files remain. Turning Pause off starts a complete reconciliation, including
changes made while paused. Refresh checks only the selected online, unpaused source.
It is disabled while that source is paused, offline or scanning.
Delete asks for confirmation, removes the source from settings and stops its
monitoring. Actual music files, ratings and other sources' memberships remain.

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
  `scanPlaylists` enables playlist discovery independently of audio extensions;
  it defaults to true in app settings and can explicitly be set to false.
- `settings.json` also stores `appearance` (`light`, `dark` or `system`) and
  `accent` (`neutral`, `violet`, `blue`, `green`, `amber`, `orange`, `coral` or
  `rose`). Missing appearance fields default to system and neutral. System mode
  follows Windows app appearance while running and keeps the chosen accent.
  On X11 it uses CUI's dark `GTK_THEME` hint when available and otherwise
  uses light.
- `settings.json` saves the master volume and ten equalizer gains in dB under
  `mixer`. The player restores both presets and custom curves from these values.
- Closing the window silences playback immediately. `state.json` stores the
  playing selection (album or library, search, filters and sort order), the current
  track ID, its slot, position and duration. On restart the player rebuilds
  the queue from the library and opens paused at that track and position.
  If the track has disappeared, the replacement at the saved slot starts at zero.
- `state.json` holds `width`, `height`, `x`, `y` and `maximized`. The
  dimensions describe the normal window in logical units. `x` and `y` are the
  top-left corner of its content in desktop coordinates: pixels on Windows
  and X11. They are `null` where the system reports no position, as on
  Wayland. FPas-Amp saves the file when its window closes and reopens the
  window there, maximized if it was. A window closed while minimized keeps
  the state it would have come back in.
- `state.json` also stores `settingsTab`, with `0` for General and `1` for Library,
  and `activePage`: `music`, `albums`, `album:<release ID>`, `artists`, `artist:<artist ID>`,
  `artist:<artist ID>:<release ID>`, `genres`, `genre:<genre ID>`, `genre:<genre ID>:<release ID>`,
  `ratings`, `settings` or `playlist:<file|local>:<UUID>`. Closing inside an album
  reopens its track list, independently of the playing track. If the catalogue
  no longer contains that release, its parent grid opens. A missing artist or genre returns to its overview. Defaults are Music and General. Malformed
  routes/tabs are repaired to these defaults. The drawer's open state is transient.
  The selected destination is saved when the window closes, including a fallback
  after the selected playlist has disappeared.

A missing file yields an empty source list or a 1000 × 680 window with no saved
position. When a file is damaged, FPas-Amp saves the original bytes in a
`*.corrupt-*.bak` sibling and writes a repaired JSON file with the recoverable
values and defaults. A saved position on a monitor that is no longer connected
is left to the system.

`library.db` stores discovered paths, source memberships, directory check states
and imported metadata
under .NET's `LocalApplicationData/fpas-amp`, independently of the music locations.
The index survives restarts and temporary source outages. Only completed directory
checks remove absent entries. SQLite may also create `library.db-wal` and
`library.db-shm` sidecars. Database errors are reported without replacing the file.

`userdata.db` beside the index holds durable track identities and user ratings
from 1 to 10, with an unassigned state. Clearing the index retains these ratings.
New paths recover an existing identity only for an unambiguous SHA-256 match
whose previous path is confirmed absent; existing copies stay separate.
Known-path tag changes retain the identity. Moving and editing together cannot
be recognized reliably by a full-file hash. Embedded covers are cached separately
on explicit request. Ratings for genres, artists, albums and tracks are stored separately in user data.

The Albums grid creates only visible rows plus a small overscan. It loads covers
on background workers, trying embedded front covers across the album's tracks first.
The workers try other embedded cover candidates before checking for `cover`,
`folder` or `front` JPG/JPEG/PNG files in an unambiguous album folder.
Missing/invalid pictures use a placeholder. Sidecar image changes are monitored.
The borderless grid fits square covers to the available width. Each tile centers
the bold album title below the cover, followed by the artist and song count on
one line. Ten native CUI stars below the caption set the album rating. Clicking
the selected star clears it.
Writes run in the background and persist in `userdata.db` across catalogue resets.
Resting the pointer on an album card for 700 ms opens native CUI hover details:
a small cover beside the title, artist and song count, with a read-only rating
below. Details reuse the loaded thumbnail and require no additional source reads.
The native status bar stays below the scrolling grid and shows the album count,
or how many of all albums match while search or filters hide some. Loading and
initial failures have their own status.

The gray album toolbar uses native CUI widgets: the search editor, the filter
popover with draft/Apply/Reset behavior, and the combined sort field/direction
menu. Artist and genre choices come from the catalogue. A new order or match set
requests covers for the newly visible albums only. Narrow windows place the two
actions below the search field.
Small content-addressed thumbnails are kept under `covers/thumbnails-v1` beside
`userdata.db`, with bounded disk/memory caches. Cached thumbnails work offline.

Playlist files in music sources are also discovered and monitored. They are
cached separately from audio entries; the scan never opens their media targets.
ATL supports M3U/M3U8, PLS, XSPF, ASX, B4S, DPL and SMIL-family files, plus
experimental FPL import. The library API can create ordered local lists, import
independent copies and export UTF-8 M3U8. Local lists are stored in `userdata.db`
and survive index resets/source removal. A playlist editor is not part of the
current settings shell. See the [library contract](src/FpasAmp.Library/README.md#playlists).
