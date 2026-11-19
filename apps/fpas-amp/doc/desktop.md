# FPas-Amp desktop application

The desktop executable loads user preferences, starts the library, restores the
saved destination and queue, and runs the CUI shell. The executable owns the
library session and awaits its disposal when the shell exits. The shell owns
the audio session and its subscriptions, including a playback session supplied
by a host or a test. For setup and user-facing behavior, see the [app README](../README.md).

## Shell and navigation

The shell builds its native view from the model, logical viewport and resolved
theme. The controller maps each child area's messages and commands into the
application model. The owner thread applies messages. Background subscriptions
publish inventory, scan progress, catalogue, playlist and playback changes.

Navigation persists stable route identities. A playlist route is validated
after the first successful summary snapshot, and falls back to Music if the
playlist has disappeared. A snapshot failure keeps the previous destinations
available. Playlist destinations currently show a summary heading and any read
warning. A window last closed on the retired Ratings page reopens on Music.

## Browsing music and collections

Music and album track lists share the track table and details view. Activating
a track uses the displayed selection and its order for the playback queue.
Artist and genre navigation scope the shared album browser by group ID. The
browser keeps grid state when entering a release and restores it when returning.
The shell reads realized ranges from native layout and posts them to Music or
the album browser, which schedule cover work for the current range.

## Editing track tags

**Edit tags** in the track's three-dot menu opens the editor in Music and in
album track lists. **Track**, **Release**, and **Credits** compare editable
local tags with individual MusicBrainz proposals. Search is explicit. Choose a
recording and release from the results, then select proposals individually or
with **Select all** and **Select none**. Editing a local field clears its
proposal selection. The heading identifies values that include saved library
edits. A process-wide gate for each connected provider enforces the interval
between requests and honors longer `Retry-After` values. MusicBrainz uses
at least one second between starts and retries HTTP 429 and 503 up to five times,
with delays of 2, 4, 8, 16 and 32 seconds. The search button and release picker
are disabled while a request or cooldown is active, even across dialog reopening.
Track and recording credits appear as soon as the recording is available. If the
release lookup fails, these values stay available for selection and saving.
For testing, the alert reports the HTTP status, reason, endpoint, attempt and
retry hint when available. [MusicBrainz API](https://musicbrainz.org/doc/MusicBrainz_API)

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
to the same file. See the [audio session contract](../../../docs/pascal/libraries/media/audio.md).

Playback triggers waveform work once rendered audio is reported. Chromaprint
fingerprinting also runs on demand from track details. Work is associated with
the current track and can be cancelled when it changes. The [library
reference](library.md#on-demand-waveforms) describes the waveform cache.

## Settings and scan control

Settings owns the source editor, appearance preferences and inventory progress
display. The Library database card sits below the source list.
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
bytes before publishing repaired data. See the [file locations and
fields](../README.md#user-files).

## Verification

The [FPAS UI tests](../tests/FpasAmp.Ui.Tests) cover state transitions,
native navigation and rendering, playlist restoration, playback and preferences.
Run them with the local SDK and native GUI prerequisites described in the
[app README](../README.md).

## Source files

<details>
<summary>Browse 84 source files</summary>

- `./`
  - [fpas-amp.slnx](../fpas-amp.slnx)
- `src/FpasAmp/`
  - [AppState.fpas](../src/FpasAmp/AppState.fpas)
  - [FpasAmp.fpasproj](../src/FpasAmp/FpasAmp.fpasproj)
  - [Main.fpas](../src/FpasAmp/Main.fpas)
  - [Settings.fpas](../src/FpasAmp/Settings.fpas)
- `src/FpasAmp/Persistence/`
  - [JsonFields.fpas](../src/FpasAmp/Persistence/JsonFields.fpas)
  - [JsonRecovery.fpas](../src/FpasAmp/Persistence/JsonRecovery.fpas)
  - [PlayerState.fpas](../src/FpasAmp/Persistence/PlayerState.fpas)
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
- `src/FpasAmp.Ui/`
  - [Fingerprints.fpas](../src/FpasAmp.Ui/Fingerprints.fpas)
  - [FpasAmp.Ui.fpasproj](../src/FpasAmp.Ui/FpasAmp.Ui.fpasproj)
  - [Shell.fpas](../src/FpasAmp.Ui/Shell.fpas)
- `src/FpasAmp.Ui/Music/`
  - [LibraryPage.fpas](../src/FpasAmp.Ui/Music/LibraryPage.fpas)
- `src/FpasAmp.Ui/Navigation/`
  - [Model.fpas](../src/FpasAmp.Ui/Navigation/Model.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Navigation/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Navigation/View.fpas)
- `src/FpasAmp.Ui/Player/`
  - [Availability.fpas](../src/FpasAmp.Ui/Player/Availability.fpas)
  - [Model.fpas](../src/FpasAmp.Ui/Player/Model.fpas)
  - [Playback.fpas](../src/FpasAmp.Ui/Player/Playback.fpas)
  - [Presets.fpas](../src/FpasAmp.Ui/Player/Presets.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/Player/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/Player/View.fpas)
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
  - [Requests.fpas](../src/FpasAmp.Ui/TagEditor/Requests.fpas)
  - [ResponseJson.fpas](../src/FpasAmp.Ui/TagEditor/ResponseJson.fpas)
  - [SearchView.fpas](../src/FpasAmp.Ui/TagEditor/SearchView.fpas)
  - [Update.fpas](../src/FpasAmp.Ui/TagEditor/Update.fpas)
  - [View.fpas](../src/FpasAmp.Ui/TagEditor/View.fpas)
- `tests/FpasAmp.Ui.Tests/`
  - [AcoustId.fpas](../tests/FpasAmp.Ui.Tests/AcoustId.fpas)
  - [AlbumBrowse.fpas](../tests/FpasAmp.Ui.Tests/AlbumBrowse.fpas)
  - [Albums.fpas](../tests/FpasAmp.Ui.Tests/Albums.fpas)
  - [CollectionFixture.fpas](../tests/FpasAmp.Ui.Tests/CollectionFixture.fpas)
  - [FpasAmp.Ui.Tests.fpasproj](../tests/FpasAmp.Ui.Tests/FpasAmp.Ui.Tests.fpasproj)
  - [NativeAlbumDetails.fpas](../tests/FpasAmp.Ui.Tests/NativeAlbumDetails.fpas)
  - [NativeAlbumMusic.fpas](../tests/FpasAmp.Ui.Tests/NativeAlbumMusic.fpas)
  - [NativeMetadataSettings.fpas](../tests/FpasAmp.Ui.Tests/NativeMetadataSettings.fpas)
  - [NativeTagEditor.fpas](../tests/FpasAmp.Ui.Tests/NativeTagEditor.fpas)
  - [MusicBrainz.fpas](../tests/FpasAmp.Ui.Tests/MusicBrainz.fpas)
  - [NativeAlbumToolbar.fpas](../tests/FpasAmp.Ui.Tests/NativeAlbumToolbar.fpas)
  - [NativeAlbums.fpas](../tests/FpasAmp.Ui.Tests/NativeAlbums.fpas)
  - [NativeCollections.fpas](../tests/FpasAmp.Ui.Tests/NativeCollections.fpas)
  - [NativeMusic.fpas](../tests/FpasAmp.Ui.Tests/NativeMusic.fpas)
  - [NativeNavigation.fpas](../tests/FpasAmp.Ui.Tests/NativeNavigation.fpas)
  - [NativePlayback.fpas](../tests/FpasAmp.Ui.Tests/NativePlayback.fpas)
  - [NativeShell.fpas](../tests/FpasAmp.Ui.Tests/NativeShell.fpas)
  - [NativeTrackNavigation.fpas](../tests/FpasAmp.Ui.Tests/NativeTrackNavigation.fpas)
  - [NativeTrackRating.fpas](../tests/FpasAmp.Ui.Tests/NativeTrackRating.fpas)
  - [Navigation.fpas](../tests/FpasAmp.Ui.Tests/Navigation.fpas)
  - [PlaybackFixture.fpas](../tests/FpasAmp.Ui.Tests/PlaybackFixture.fpas)
  - [Player.fpas](../tests/FpasAmp.Ui.Tests/Player.fpas)
  - [Preferences.fpas](../tests/FpasAmp.Ui.Tests/Preferences.fpas)
  - [StatusCounts.fpas](../tests/FpasAmp.Ui.Tests/StatusCounts.fpas)
  - [TrackFilters.fpas](../tests/FpasAmp.Ui.Tests/TrackFilters.fpas)
  - [TrackRating.fpas](../tests/FpasAmp.Ui.Tests/TrackRating.fpas)

</details>
