# FPas-Amp desktop application

The desktop executable loads user preferences, starts the library, restores the
saved destination and queue, and runs the CUI shell. The executable owns the
library session and awaits its disposal when the shell exits. The shell owns
the audio session and its subscriptions, including a playback session supplied
by a host or a test. For setup and user-facing behavior, see the
[documentation index](README.md).

## Shell and navigation

The shell builds its native view from the model, logical viewport and resolved
theme. The controller maps each child area's messages and commands into the
application model. The owner thread applies messages. Background subscriptions
publish inventory, scan progress, catalogue, playlist and playback changes.

Navigation persists stable route identities. A playlist route is validated
after the first successful summary snapshot, and falls back to Music if the
playlist has disappeared. A snapshot failure keeps the previous destinations
available. A window last closed on the retired Ratings page reopens on Music.

## Browsing music and collections

Music, albums and playlists share a virtual, sortable track table and the
details view. Activating a track uses the displayed selection and its order
for the playback queue.
In Music and playlists, the **Artist**, **Album** and **Genre** links open
their catalogue pages. The Back button returns to the originating table.
An album link opens the indexed release even if other albums share its name.
Artist pages include releases that credit the artist as a track performer.

During a session, Music, each playlist, and each album track view retain their
search, applied filters, sort, row selection and vertical position. These
controls are restored when the view is reopened and applied to the latest
catalogue rows.
The minimum rating uses a native step slider. Zero accepts any rating.
Selecting **Unrated** disables the slider and matches items without a rating.
Artist and genre navigation scope the shared album browser by group ID. The
browser keeps each cover grid's filters while navigating and records its own
scroll position in `state.json`. This applies to Albums, Artists, Genres, and
the albums within each artist or genre. The cover scroll area spans the main
pane, including both gutters, while the cards line up with the search toolbar.
The shell reads realized ranges from native layout and posts them to Music or
the album browser, which schedule cover work for the current range.
A playlist starts in its stored order and retains repeated tracks. Its table
lists entries that resolve to indexed files. Missing files and remote playlist
entries stay in storage and are counted in the total. The playback bookmark
records the playlist key and selection so startup can rebuild its queue.

## Editing track tags

**Edit tags** in the track's three-dot menu opens the editor in Music and in
album track lists. **Track**, **Release**, and **Credits** compare editable
local tags with individual MusicBrainz proposals. Click **Search MusicBrainz**
or press Enter in the search field. Use its clear button to reset the query.
Choose a recording and release from the results, then select proposals individually or
with **Select all** and **Select none**. Editing a local field clears its
proposal selection. The heading identifies values that include saved library
edits. A process-wide gate for each connected provider enforces the interval
between requests and honors longer `Retry-After` values. The
[MetaBrainz.MusicBrainz](https://github.com/Zastai/MetaBrainz.MusicBrainz) client
handles MusicBrainz searches and lookups. MusicBrainz uses at least one second
between starts. Microsoft HTTP Resilience retries HTTP 429 and 503 up to five times,
with delays of 2, 4, 8, 16 and 32 seconds. The search button and release picker
are disabled while a request or cooldown is active, even across dialog reopening.
Track and recording credits appear as soon as the recording is available. If the
release lookup fails, these values stay available for selection and saving.
For testing, the alert reports the HTTP status, reason, endpoint, attempt and
retry hint when available. [MusicBrainz API](https://musicbrainz.org/doc/MusicBrainz_API)

Native AOT publishing enables the .NET JSON reflection setting for
MetaBrainz's registered converters. The provider's search, recording and
release paths can run with this setting. The linker still reports trimming
and AOT analysis warnings for MetaBrainz and other app dependencies.

**Scan AcoustID** identifies the selected track by audio. It needs the application
key from **Settings → Metadata**. It reuses the cached Chromaprint and full file
duration or measures and caches them first. The scan sends the application key,
fingerprint and duration to AcoustID and lists MusicBrainz recording and release
matches for explicit selection. AcoustID has its own process-wide interval of
334 ms. [AcoustID API](https://acoustid.org/webservice#lookup-by-fingerprint)

**Custom** lists editable name and value pairs and selectable MusicBrainz
fields such as barcode, ASIN, script and release IDs. Proposals use the file's
existing names when they match. Only the list scrolls. **Add tag** moves to the
new name field. The dialog keeps the tabs, column headings,
destination checkboxes, and save controls in place. The database is selected
by default and writing the audio file requires selecting **Audio file**.
Multiline values open in a text area. Binary ID3 frames display their owner
and a read-only identifier or hex preview. Saving other tags preserves their
bytes. Values above 65,535 UTF-8 bytes also stay intact.
The [library reference](library.md#tag-editing) describes both destinations.

## Playback and analysis

The player reducer submits transport requests through one FIFO mailbox. One
worker performs file and audio device operations and a separate subscription
forwards device events to the UI. The reducer checks the track path and request
revision before accepting an event. These checks distinguish an old seek or
natural end from observations for the current selection, including repeat visits
to the same file. See the [audio session contract](media/audio.md).

Playback triggers waveform work once rendered audio is reported. Chromaprint
fingerprinting also runs on demand from track details. Work is associated with
the current track and can be cancelled when it changes. The [library
reference](library.md#on-demand-waveforms) describes the waveform cache.

The equalizer preset button opens a native menu. The menu checks the applied
preset. Select a preset to apply it or press Escape to close the menu.

## Online radio

**Radio → Stations** searches the public Radio Browser directory by name,
country, language and genre. Searches are cancellable, try another mirror on
failure and show up to 100 matching stations. Results arrive independently of
the music library. [Radio Browser API](https://docs.radio-browser.info/)

Star a directory result or use **Add stream URL** to save an HTTP or HTTPS
station under Radio. Favorites are stored in order in `settings.json` and can
be opened and played without querying the directory again. A favorite's page
shows available station details and the on-air title when supplied by the
stream. The sidebar lists local music playlists below Radio.

Radio playback uses FFmpeg to decode direct audio and HLS streams. The
[Windows portable build](#windows-portable-build) includes FFmpeg. The app looks
for `ffmpeg` on `PATH` when a bundled copy is unavailable.
The player shares its equalizer, volume and stereo meters with local music.
It shows the live or buffering state in place of the waveform. Stop closes the
stream. Press Play after a disconnect to reconnect. Streams do not start
automatically when the app opens.

## Settings and scan control

Settings owns the source editor, appearance preferences and inventory progress
display. The Library database card sits below the source list.
The General tab uses the native Light, Dark and System selector and named
accent buttons.
Accepted source settings are saved before the library is reconfigured.
A confirmed index clear or source reset saves the corresponding pause settings
before requesting the clear. The settings view waits for the worker's result
before releasing the clearing control. See the [library control
contract](library.md#lifetime-and-control).

The AcoustID card on the Metadata tab has optional application and user API keys. The
fields mask their contents and link to the corresponding AcoustID setup pages.
**Save keys** writes the values locally in plain text to `settings.json`.
Clear a field and save to remove its key.

## Startup and user files

The executable reads preferences and window state from separate JSON files.
It uses the saved queue selection to rebuild an available queue. Restored
playback stays paused and the shell opens the device when playback is requested.
If the queue cannot be read at startup and no new queue is selected, closing
preserves the existing bookmark for the next launch.

Closing silences the device before joining GUI effects. When the final audio
snapshot matches the player's track and revision, its rendered position is
saved. Otherwise the player retains its last selected position, which can
include a seek still in flight. The shell also saves pending mixer changes.

The JSON store validates values before publishing a replacement and serializes
file access with a per-user mutex. When reading a damaged document, it recovers
usable fields, falls back to defaults where necessary and backs up the original
bytes before publishing repaired data. See [user files and recovery](user-files.md).

## Windows portable build

On Windows x64, run the [compiler distribution script](../../../dist-small.ps1)
to prepare the matching SDK and CUI packages, then run
[the FPas-Amp distribution script](../dist.ps1).
It publishes the app with Native AOT and packages it as `bin/FpasAmp.exe`.
The [full distribution](../../../docs/build/distribution.md#distribute-the-compiler-and-applications)
runs both steps.
Copy that one file to a writable directory. The launcher verifies and
extracts the app, native DLLs, `fpcalc.exe` and the radio decoder to a
versioned directory under `FPasAmp-runtime/` beside it.

The [dependency packaging script](../packaging/windows/Dependencies.ps1)
uses the pinned [Windows x64 LGPL build of FFmpeg](https://github.com/FFmpeg-Builds/FFmpeg-Builds/releases/tag/autobuild-2026-11-20-12-40).
It includes the upstream license, matching FFmpeg source and build recipe,
and the app-local Visual C++ runtime needed by the audio DLL. The portable
app finds its bundled FFmpeg by its absolute path.

## Windows desktop launch

Local `dotnet build` and `dotnet run` use the Windows GUI apphost. To open the
app without a terminal window, launch `FpasAmp.exe` from the build directory.
The executable embeds the [Windows icon](../packaging/windows/FpasAmp.ico),
rendered from the [desktop SVG](../packaging/linux/io.github.tobiasbick.FpasAmp.svg).
The native GUI uses the embedded icon for the window caption.

## Linux desktop installation

Linux x64 publication with an executable includes an application-menu entry and
a scalable icon under `share/`. The launcher calls `FpasAmp` by name. Keep the
published directory together and put its launcher on the desktop session's
`PATH`, for example through a symlink in `~/.local/bin`. The `apphost` mode below
requires the .NET 10 runtime.

From the FPas-Amp directory, publish and install for the current user:

```sh
app_dir="$HOME/.local/opt/fpas-amp"
data_dir="${XDG_DATA_HOME:-$HOME/.local/share}"
app_id=io.github.tobiasbick.FpasAmp

install -d "$HOME/.local/opt" "$HOME/.local/bin"
fpas publish src/FpasAmp/FpasAmp.fpasproj --runtime linux-x64 --mode apphost --output "$app_dir"
ln -s "$app_dir/FpasAmp" "$HOME/.local/bin/FpasAmp"
install -Dm644 "$app_dir/share/applications/$app_id.desktop" "$data_dir/applications/$app_id.desktop"
install -Dm644 "$app_dir/share/icons/hicolor/scalable/apps/$app_id.svg" "$data_dir/icons/hicolor/scalable/apps/$app_id.svg"
```

The desktop file's name, `StartupWMClass`, icon basename and GUI `AppId` use
`io.github.tobiasbick.FpasAmp`. On X11, `xprop WM_CLASS` on a running window
reports this identity. With grouping enabled, Xfce lists multiple FPas-Amp
windows under one task button. Close a resized window normally to save its
dimensions in `state.json` for the next launch.

- [Application-menu entry](../packaging/linux/io.github.tobiasbick.FpasAmp.desktop)
- [Scalable application icon](../packaging/linux/io.github.tobiasbick.FpasAmp.svg)

## Verification

The [FPAS UI tests](../tests/FpasAmp.Ui.Tests) cover state transitions,
native navigation and rendering, playlist restoration, playback and preferences.
Run them with the local SDK and native GUI prerequisites described in the
[app README](../README.md).

## Source files

<details>
<summary>Browse source files</summary>

- `./`
  - [fpas-amp.slnx](../fpas-amp.slnx)
- `src/FpasAmp/`
  - [AppState.fpas](../src/FpasAmp/AppState.fpas)
  - [FpasAmp.fpasproj](../src/FpasAmp/FpasAmp.fpasproj)
  - [Main.fpas](../src/FpasAmp/Main.fpas)
  - [Settings.fpas](../src/FpasAmp/Settings.fpas)
- `src/FpasAmp/Persistence/`
  - [JsonFields.fpas](../src/FpasAmp/Persistence/JsonFields.fpas)
  - [JsonOutput.fpas](../src/FpasAmp/Persistence/JsonOutput.fpas)
  - [JsonRecovery.fpas](../src/FpasAmp/Persistence/JsonRecovery.fpas)
  - [PlayerState.fpas](../src/FpasAmp/Persistence/PlayerState.fpas)
  - [RadioFavorites.fpas](../src/FpasAmp/Persistence/RadioFavorites.fpas)
  - [UserFiles.fpas](../src/FpasAmp/Persistence/UserFiles.fpas)
- `src/FpasAmp.Ui/Albums/`
  - [Artwork.fpas](../src/FpasAmp.Ui/Albums/Artwork.fpas)
  - [Browse.fpas](../src/FpasAmp.Ui/Albums/Browse.fpas)
  - [Details.fpas](../src/FpasAmp.Ui/Albums/Details.fpas)
  - [Grid.fpas](../src/FpasAmp.Ui/Albums/Grid.fpas)
  - [Groups.fpas](../src/FpasAmp.Ui/Albums/Groups.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Albums/Model.fpas)
  - [RatingStyle.fpas](../src/FpasAmp.Ui/Albums/RatingStyle.fpas)
  - [Status.fpas](../src/FpasAmp.Ui/Albums/Status.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Albums/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Albums/View.fpas)
- `src/FpasAmp.Ui/Albums/Music/`
  - [Browse.fpas](../src/FpasAmp.Ui/Albums/Music/Browse.fpas)
  - [Filters.fpas](../src/FpasAmp.Ui/Albums/Music/Filters.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Albums/Music/Model.fpas)
  - [Ratings.fpas](../src/FpasAmp.Ui/Albums/Music/Ratings.fpas)
  - [TrackTable.fpas](../src/FpasAmp.Ui/Albums/Music/TrackTable.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Albums/Music/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Albums/Music/View.fpas)
- `src/FpasAmp.Ui/Albums/Music/Details/`
  - [Metadata.fpas](../src/FpasAmp.Ui/Albums/Music/Details/Metadata.fpas)
  - [Source.fpas](../src/FpasAmp.Ui/Albums/Music/Details/Source.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Albums/Music/Details/View.fpas)
- `src/FpasAmp.Ui/Albums/Toolbar/`
  - [Filters.fpas](../src/FpasAmp.Ui/Albums/Toolbar/Filters.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Albums/Toolbar/Model.fpas)
  - [Sorting.fpas](../src/FpasAmp.Ui/Albums/Toolbar/Sorting.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Albums/Toolbar/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Albums/Toolbar/View.fpas)
- `src/FpasAmp.Ui/App/`
  - [Analysis.fpas](../src/FpasAmp.Ui/App/Analysis.fpas)
  - [Browsing.fpas](../src/FpasAmp.Ui/App/Browsing.fpas)
  - [Controller.fpas](../src/FpasAmp.Ui/App/Controller.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/App/Model.fpas)
  - [Radio.fpas](../src/FpasAmp.Ui/App/Radio.fpas)
  - [Subscriptions.fpas](../src/FpasAmp.Ui/App/Subscriptions.fpas)
- `src/FpasAmp.Ui/`
  - [Fingerprints.fpas](../src/FpasAmp.Ui/Fingerprints.fpas)
  - [FpasAmp.Ui.fpasproj](../src/FpasAmp.Ui/FpasAmp.Ui.fpasproj)
  - [Shell.fpas](../src/FpasAmp.Ui/Shell.fpas)
- `src/FpasAmp.Ui/Collections/`
  - [FilterOptions.fpas](../src/FpasAmp.Ui/Collections/FilterOptions.fpas)
  - [TableRows.fpas](../src/FpasAmp.Ui/Collections/TableRows.fpas)
  - [TableSort.fpas](../src/FpasAmp.Ui/Collections/TableSort.fpas)
- `src/FpasAmp.Ui/Music/`
  - [LibraryPage.fpas](../src/FpasAmp.Ui/Music/LibraryPage.fpas)
  - [Playlist.fpas](../src/FpasAmp.Ui/Music/Playlist.fpas)
- `src/FpasAmp.Ui/Navigation/`
  - [Model.fpas](../src/FpasAmp.Ui/Navigation/Model.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Navigation/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Navigation/View.fpas)
- `src/FpasAmp.Ui/Player/`
  - [Availability.fpas](../src/FpasAmp.Ui/Player/Availability.fpas)
  - [Live.fpas](../src/FpasAmp.Ui/Player/Live.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Player/Model.fpas)
  - [Playback.fpas](../src/FpasAmp.Ui/Player/Playback.fpas)
  - [Presets.fpas](../src/FpasAmp.Ui/Player/Presets.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Player/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Player/View.fpas)
- `src/FpasAmp.Ui/Radio/`
  - [Directory.fpas](../src/FpasAmp.Ui/Radio/Directory.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Radio/Model.fpas)
  - [Toolbar.fpas](../src/FpasAmp.Ui/Radio/Toolbar.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Radio/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Radio/View.fpas)
- `src/FpasAmp.Ui/Settings/`
  - [DatabaseActions.fpas](../src/FpasAmp.Ui/Settings/DatabaseActions.fpas)
  - [Editing.fpas](../src/FpasAmp.Ui/Settings/Editing.fpas)
  - [Metadata.fpas](../src/FpasAmp.Ui/Settings/Metadata.fpas)
  - [Editor.fpas](../src/FpasAmp.Ui/Settings/Editor.fpas)
  - [General.fpas](../src/FpasAmp.Ui/Settings/General.fpas)
  - [InventoryState.fpas](../src/FpasAmp.Ui/Settings/InventoryState.fpas)
  - [Library.fpas](../src/FpasAmp.Ui/Settings/Library.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Settings/Model.fpas)
  - [ScanStatus.fpas](../src/FpasAmp.Ui/Settings/ScanStatus.fpas)
  - [SourceActions.fpas](../src/FpasAmp.Ui/Settings/SourceActions.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Settings/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Settings/View.fpas)
- `src/FpasAmp.Ui/Providers/`
  - [RequestPacing.fpas](../src/FpasAmp.Ui/Providers/RequestPacing.fpas)
- `src/FpasAmp.Ui/TagEditor/`
  - [AcoustId.fpas](../src/FpasAmp.Ui/TagEditor/AcoustId.fpas)
  - [Changes.fpas](../src/FpasAmp.Ui/TagEditor/Changes.fpas)
  - [CustomTags.fpas](../src/FpasAmp.Ui/TagEditor/CustomTags.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/TagEditor/Model.fpas)
  - [MusicBrainz.fpas](../src/FpasAmp.Ui/TagEditor/MusicBrainz.fpas)
  - [MusicBrainzTags.fpas](../src/FpasAmp.Ui/TagEditor/MusicBrainzTags.fpas)
  - [Requests.fpas](../src/FpasAmp.Ui/TagEditor/Requests.fpas)
  - [ResponseJson.fpas](../src/FpasAmp.Ui/TagEditor/ResponseJson.fpas)
  - [SearchView.fpas](../src/FpasAmp.Ui/TagEditor/SearchView.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/TagEditor/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/TagEditor/View.fpas)
- `tests/FpasAmp.Ui.Tests/`
  - [FpasAmp.Ui.Tests.fpasproj](../tests/FpasAmp.Ui.Tests/FpasAmp.Ui.Tests.fpasproj)
- `tests/FpasAmp.Ui.Tests/Albums/`
  - [AlbumBrowse.fpas](../tests/FpasAmp.Ui.Tests/Albums/AlbumBrowse.fpas)
  - [Albums.fpas](../tests/FpasAmp.Ui.Tests/Albums/Albums.fpas)
  - [NativeAlbumDetails.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeAlbumDetails.fpas)
  - [NativeAlbumMusic.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeAlbumMusic.fpas)
  - [NativeAlbumToolbar.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeAlbumToolbar.fpas)
  - [NativeAlbums.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeAlbums.fpas)
  - [NativeMusic.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeMusic.fpas)
  - [NativeTrackNavigation.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeTrackNavigation.fpas)
  - [NativeTrackLinks.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeTrackLinks.fpas)
  - [NativeTrackRating.fpas](../tests/FpasAmp.Ui.Tests/Albums/NativeTrackRating.fpas)
  - [TrackFilters.fpas](../tests/FpasAmp.Ui.Tests/Albums/TrackFilters.fpas)
  - [TrackRating.fpas](../tests/FpasAmp.Ui.Tests/Albums/TrackRating.fpas)
- `tests/FpasAmp.Ui.Tests/Player/`
  - [NativePlayback.fpas](../tests/FpasAmp.Ui.Tests/Player/NativePlayback.fpas)
  - [PlaybackFixture.fpas](../tests/FpasAmp.Ui.Tests/Player/PlaybackFixture.fpas)
  - [PlayerQueue.fpas](../tests/FpasAmp.Ui.Tests/Player/PlayerQueue.fpas)
  - [PlayerTransport.fpas](../tests/FpasAmp.Ui.Tests/Player/PlayerTransport.fpas)
- `tests/FpasAmp.Ui.Tests/Providers/`
  - [AcoustId.fpas](../tests/FpasAmp.Ui.Tests/Providers/AcoustId.fpas)
  - [MusicBrainz.fpas](../tests/FpasAmp.Ui.Tests/Providers/MusicBrainz.fpas)
  - [NativeMetadataSettings.fpas](../tests/FpasAmp.Ui.Tests/Providers/NativeMetadataSettings.fpas)
  - [NativeRadio.fpas](../tests/FpasAmp.Ui.Tests/Providers/NativeRadio.fpas)
  - [NativeTagEditor.fpas](../tests/FpasAmp.Ui.Tests/Providers/NativeTagEditor.fpas)
  - [ProviderHttp.fpas](../tests/FpasAmp.Ui.Tests/Providers/ProviderHttp.fpas)
- `tests/FpasAmp.Ui.Tests/Shell/`
  - [CollectionFixture.fpas](../tests/FpasAmp.Ui.Tests/Shell/CollectionFixture.fpas)
  - [NativeCollections.fpas](../tests/FpasAmp.Ui.Tests/Shell/NativeCollections.fpas)
  - [NativeNavigation.fpas](../tests/FpasAmp.Ui.Tests/Shell/NativeNavigation.fpas)
  - [NativeShell.fpas](../tests/FpasAmp.Ui.Tests/Shell/NativeShell.fpas)
  - [Navigation.fpas](../tests/FpasAmp.Ui.Tests/Shell/Navigation.fpas)
  - [Preferences.fpas](../tests/FpasAmp.Ui.Tests/Shell/Preferences.fpas)
  - [StatusCounts.fpas](../tests/FpasAmp.Ui.Tests/Shell/StatusCounts.fpas)

</details>
