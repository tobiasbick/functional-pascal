# FpasAmp.Library

UI-independent FPAS directory discovery, audio metadata, playlists and durable user
ratings, using .NET, `Microsoft.Data.Sqlite` and the [Fpas.Media.Tags](../../../../src/Fpas.Media.Tags/README.md) library. Discovery enumerates
filesystem entries; the optional second stage reads tags, technical audio
properties and SHA-256 without decoding or modifying audio.

## Acoustic fingerprints

`Metadata.Session.FingerprintAsync(track, generate, cancellation)` reads the
catalogue cache. Passing `true` also runs the packaged Chromaprint `fpcalc` on
a cache miss. The cache stores the full encoded fingerprint, algorithm number
and duration reported by the decoder, keyed by the file's SHA-256 and algorithm.
Switching files or closing the app can cancel the child process. The inventory
and metadata scans do not enqueue fingerprint work.

## Lifetime and control

Call `Inventory.Monitoring.Session.Start` with `Sources.MusicSource` settings,
an absolute catalogue database path and a parent cancellation token.
`Persistence.Paths.DatabasePath` resolves the app's default `library.db`.
Pass the distinct absolute `UserDataPath` argument to enable metadata/ratings;
`Persistence.Paths.UserDataPath` resolves the default `userdata.db`.
Without that argument the session performs presence discovery only and supports
an isolated `:memory:` catalogue for tests. Start validates/copies configuration;
filesystem/database operations run on background workers.

Await `DisposeAsync` on every exit, normally with `await using const`. It cancels
and joins both stages and releases watchers/connections. `Completion` exposes
discovery failures. The enabled, borrowed `Metadata` handle exposes its own
`Ready` and `Completion`; do not dispose it separately from the inventory owner.

- `Configure` replaces settings without waiting for a scan. Pending changes
  collapse to the latest configuration; names/equivalent extensions do not rescan.
- `Refresh()` requests a complete check; `Refresh(SourceId)` targets one active
  source. Paused/offline sources must first be resumed.
- Pause/offline retain source-owned entries while other sources continue. Resume
  reconciles changes made while suspended without retaining an event history.
- `ClearAsync(SourceId, Cancellation)` pauses/clears one source; empty ID means
  all sources. The returned snapshot acknowledges a committed transaction.
  The UI saves pause preferences first. Duplicate targets share acknowledgement;
  at most sixteen distinct clear requests queue. Caller cancellation stops waiting
  while an accepted operation remains owned by the session.
- Removing a source means saving a configuration without that ID and calling
  `Configure`. Its membership/watchers are released. Shared memberships, media
  files and durable ratings remain. Index clearing also retains ratings.

`Changes` publishes detached presence snapshots with revision/generation,
per-source availability, immutable paths, directory errors and watcher coverage.
Its capacity is one: slow readers receive the latest completed view. SQLite
startup views are explicitly cached until reconciled. Matching an extension
does not establish valid audio; metadata failures are recorded per file.

`Progress` is a separate capacity-one stream with queued/scanning/idle/paused/
offline phases and actual file/directory counters, normally at most ten updates
per second plus state transitions. It avoids full path-list copies and invented
percentages. These counters describe directory discovery, independently of tags.
Drain remaining snapshots before awaiting a channel reader's completion.

## Reconciliation and persistence

One discovery worker streams each directory with `EnumerateFileSystemInfos`.
Hidden directories are included; reparse points and roots reached through
links/junctions are excluded. Paths compare ordinally on Unix and without case
on Windows. Hard links and symbolic aliases are not merged.

Equal/nested active roots share traversal/watchers, while source IDs retain their
own filtered membership/error views. `ScanPlaylists` selects playlist discovery
independently of audio extensions. With neither selected, a source schedules no work.
Watchers are enabled before scanning; callbacks only invalidate directories.
Events are hints: changes are reconciled from the filesystem, new directories
are traversed, and overflow/saturation requests a complete check/reinstallation.
Failed enumeration and unavailable roots retain earlier entries. Confirmed
complete directory checks alone authorize deletion. Interrupted work needed by
other active sources is requeued.

Defaults: one discovery worker, a fixed **500 ms** batching window, **1,024**
pending directories, and a full control scan **15 minutes** after the previous
full check. `Inventory.Model.Options` configures these bounds. Pending work is
bounded; stored inventories necessarily scale with files/directories.

`settings.json` owns source configuration. The reconstructible `library.db` holds:

- `sources`, `source_files`: source/root context and shared file memberships.
- `files`: normalized path, stable index ID, directory, byte length and UTC
  modification ticks. Index IDs survive unchanged paths, not index deletion.
- `directories`: per-source directory context, last successful check and error.
- `metadata`: last successful tag JSON, read error/time, attempted freshness,
  SHA-256, durable user UUID and indexed sort fields.
- `waveforms` stores 1,024 normalized loudness samples per played recording,
  keyed by the indexed SHA-256. Duplicate files share the same envelope.
- `metadata_people`, `metadata_genres`: ordered multi-value relationships with
  indexed invariant case keys. ATL's public separator convention is used;
  artist names are not split on commas or slashes.
- `playlist_files`, `playlist_items`: source-owned playlist cache, ordered entries,
  retained references, durable track associations and latest read errors.
- `content_jobs`: indexed pending audio/playlist work, requeued by file-stamp changes.
- `album_tracks`: indexed release memberships and sort/directory context, derived
  transactionally from successful tags. Cover sidecar observations use `files.kind=2`
  and never contribute to audio counters or metadata/hash jobs.

Connections use WAL, foreign keys, prepared/bound SQL and full SQLite durability.
Each directory/tag write is transactional; obsolete generations/cancellation
roll back the current checkpoint. The metadata worker retries busy writes with
short SQLite timeouts and cancellation-aware backoff. A queued file stays in the
catalogue until its result commits, so interrupted work resumes on the next start.
Orphans are collected only after their last
source association disappears. Existing presence-only catalogues are upgraded
without replacing rows. Corrupt/unsupported stores fail without being reset.
The index contains no irreplaceable user data.

Index/user database files, their sidecars and the adjacent cover cache are excluded
from watcher notifications. Runtime database files/sidecars are ignored by Git.
Native SQLite assets arrive through NuGet; ATL's managed dependency detects text encodings.

## Metadata and queries

The enabled `Metadata` stage reads one file at a time using a **64 KiB** hash
buffer. Pending work stays in an indexed SQLite queue; each iteration processes
at most one playlist and one audio file. Successful unchanged size/time observations skip hashing/tag reads.
Files are revalidated around reading and at commit. Failures retry after a minute
or an observed size/time change; earlier successful tags are retained.

[Metadata.Model](Metadata/Model.fpas) defines the selected fields: title/subtitle,
performers, album artists, album, genres, composers, dates, track/disc positions
and totals, explicit sort tags, compilation, comment, grouping, label/copyright,
BPM, MusicBrainz identities, duration, codec, bitrate, sample rate, channels,
bit depth and ReplayGain. Dates use ATL's normalized representation, including
its interpretation of year-only tags. MP3/ID3, Ogg/Vorbis and Ogg/Opus are tested
with real synthetic containers; ATL also supports the app's FLAC/M4A locations.

`QueryAsync` combines source, album, performer, genre and minimum-rating filters,
ordering by title/album/artist/album artist/genre/date/duration/rating/path, and
pages of at most **500** rows. Results include path, index ID, durable UUID,
fingerprint, last successful tags, current error and user rating. Equal album
titles are not assumed to identify the same release.

Pause/removal/shutdown interrupt hashing between blocks. Blocking OS calls and
ATL's synchronous parser must finish before that operation can exit. The UI
thread remains free; per-file read errors remain visible in query results and
fatal lifetime failures are reported by the shell.

## Durable identities and ratings

`userdata.db` stores UUIDs, latest paths/fingerprints and nullable integer ratings.
`SetRatingAsync(TrackId, Value, Cancellation)` accepts **1–10**; null clears it.
Imported file ratings never overwrite user choices. Clearing/removing/rebuilding
the catalogue retains this database. The Albums destination opens a Music view
with the release's track list and independent track rating controls. UI bindings use the existing
native `Ratings.Rating` with `Maximum := 10`, mapping its cleared value 0 to null.

Full-file SHA-256 is independent of the path but changes with tag edits. Known
paths retain UUID/rating while updating their fingerprint. A new path recovers
an identity only for exactly one hash/size candidate whose old path is confirmed
absent and whose known source is not paused, offline or unreadable. Existing
copies and ambiguous duplicates receive separate UUIDs. Simultaneous moving and
editing is not reliably recognizable; replacing content at an existing path is
treated as an update. This is file recognition, not an acoustic fingerprint.

`CoverAsync(FileId, Cancellation)` prefers an embedded front cover, then the first
other picture, only on demand
and caches it under `covers` beside the user database, addressed by SHA-256.
Scans leave image data unloaded. Cached artwork works offline; absence is cached
too. Images retain their original MIME type/bytes, limited to **sixteen MiB**.
Cache writes replace temporary files atomically; the cache is reconstructible.

## Search

`Search.Matching` ranks prepared documents for free text the way a modern search
box does, without query syntax: `*`, `?` and regular expressions have no meaning.
`Document.Create` takes weighted fields, `Query.Parse` the typed text, and `Rank`
returns one score per document, zero for no match.

- Words match in any order, and a document needs only one of them. Documents with
  more of the words rank higher, and words that many documents share, such as
  "the", count less.
- Case, accents and apostrophes do not matter: "Björk" finds "bjork", "Straße"
  finds "strasse" and "Don't" finds "dont". Adjacent words are also indexed
  together, so "pinkfloyd" finds "Pink Floyd".
- The start of a word matches while it is being typed. Words of four letters or
  more tolerate one typo, eight or more two, including swapped neighbors.
- `Search.Text.Phonetic` gives the Kölner Phonetik code, so "Mayer" finds "Meier".
- Numbers such as years match exactly or as a prefix, never as a near miss.
- Matches far below the best one are dropped as noise.

## Albums and artwork

The borrowed `Metadata.Albums` handle publishes sorted release headers and the indexed
audio count through `ObserveAsync`. The count includes tracks awaiting metadata.
`ThumbnailAsync(Album, Cancellation)` returns one immutable RGBA thumbnail.
`SetRatingAsync(Id, Value, Cancellation)` stores an independent album rating from
one to ten, or null to clear it. Ratings live in `userdata.db`, survive catalogue
resets/rescans, and never change track ratings or source tags. Header observations
include user-data changes without invalidating unchanged artwork.
Headers also carry the distinct genre tags of the album's tracks.
Header observations run only while the Albums slice is open. They poll database
commit versions and publish changed snapshots. The catalogue derives memberships
without reopening audio when upgrading cached successful tags; discovery starts
after that schema initialization completes.

A valid MusicBrainz release ID joins a release across folders. Without one, identity
uses album title, album artist (performer or compilation fallback), release date and
normalized album folder. Recognized CD/disc subfolders share their parent when disc
tags are present. Missing album titles use that folder's name. Equal display names
alone never merge unrelated releases.

Artwork resolution first tries tagged front covers across album members, then
tries the remaining embedded cover candidates across those members. It next
tries local `cover`, `folder`, `front` names, each
in JPG/JPEG/PNG order, case-insensitively. A sidecar folder must belong to exactly one
indexed album; mixed folders cannot give several releases the same accidental image.
Only observed files inside active sources are opened. Invalid/missing art becomes
a placeholder. Changed or deleted sidecars change the album revision; missing
results do not prevent subsequently added pictures from being discovered.

The GUI realizes native CUI virtual rows containing native grids/cards/images and
requests thumbnails after native layout reports its range. Row factories remain
pure. At most two source requests run for that range. Changing the range, leaving
Albums or closing the window cancels work; generation checks reject stale results.
The view retains only the requested images. Cached positive thumbnails work offline.
All source reads, decoding and resizing stay outside the GUI thread.

[Fpas.Media.Images](../../../../src/Fpas.Media.Images/README.md) decodes the artwork.
[FpasAmp.Imaging](../FpasAmp.Imaging/README.md) owns content-addressed thumbnail storage: 384 × 384 RGBA, 32 MiB decoded cache and
256 MiB disk pixel cache. Shared picture bytes share a blob across albums. Album
thumbnail requests bypass the per-track original-image cache to avoid duplication.

## Playlists

The borrowed `Metadata.Playlists` handle provides asynchronous `ListAsync`,
`ReadAsync`, `SaveAsync`, `DeleteAsync`, `ImportAsync` and `ExportAsync` operations.
It shares initialization/cancellation with the owning session and starts no extra
worker. All file/database I/O stays off the caller's thread.

`ObserveAsync(Publish, Cancellation)` sends complete immutable summary snapshots,
including an initially empty result. It observes both database commit versions
twice per second and reloads summaries only after changes. Equal snapshots are
not republished; pagination is revalidated against concurrent writes. Cancellation
joins the observation and releases its connections. The caller owns that lifetime.

Sources with `ScanPlaylists := true` discover playlists recursively and reconcile
watcher changes alongside audio. The app enables this by default; the core source
type defaults to false for callers requesting audio presence only. Playlist files
never contribute to audio path lists or scan file counters. Their parsed cache
follows source membership, pause/offline, reset and deletion rules. The source file
is never rewritten by scanning.

ATL imports M3U/M3U8, PLS, XSPF, ASX/WAX/WVX, B4S, DPL and
SMIL/SMI/WPL/ZPL. FPL import uses ATL's experimental URI extractor and cannot
guarantee support for every Foobar2000 binary variant. Export is exclusively
**M3U8 UTF-8**. The vendored ATL changes disable referenced-media metadata reads,
make parse failures observable and correct documented playlist parser defects.

[Playlists.Model](Playlists/Model.fpas) distinguishes source-file and local keys.
Order and duplicates are retained, including unavailable paths and remote URIs.
Relative references resolve against the playlist directory; URI escaping is decoded
for XML formats while ordinary M3U paths retain literal percent characters.
Foreign-platform absolute paths are retained without a host catalogue association.
No referenced folder is enrolled automatically and no remote target is fetched.
Known tracks bind by durable UUID; late indexing resolves previously unknown local
references. Reads follow recognized moves through that UUID. `Indexed` means a
catalogue membership exists, including cached/offline entries, rather than proving
the file is currently readable.

`SaveAsync(Id, Name, Items, Cancellation)` creates a durable local list when `Id`
is null and replaces an existing local list otherwise. Items contain either
`Target.Track(UUID)` or `Target.Location(AbsolutePathOrUri)` plus an optional title.
Replacement is transactional and preserves the supplied order/duplicates. Unknown
local IDs/track UUIDs and multiline references/titles are rejected. `DeleteAsync`
only removes a local list, retaining tracks/ratings. Local lists live in
`userdata.db` and survive source removal, index reset and rebuilding.

`ImportAsync(Path, Cancellation, Name)` explicitly copies a supported playlist
into a new local list, independently of sources and the input file's lifetime.
`ExportAsync(Key, Destination, Cancellation, Overwrite)` writes any cached/local
list to an absolute `.m3u8` path using current track paths. Local paths are
written relative to the destination directory when possible. It writes a temporary
file in the destination directory and renames it after success; replacement
requires `Overwrite := true`. Missing entries are retained. When an entry has
no title, the export uses its filename without the extension.

Summary queries have at most **200** rows and support store/source filters.
Parsing uses a bounded temporary snapshot: **sixteen MiB** and **100000** entries
per list. Failed reads retain the last successful entries and retry after one
minute or a stamp change. Cancellation interrupts copying/entry iteration and
checks both sides of ATL's synchronous parser; it cannot interrupt that parser
mid-call. Durable list mutations roll back before commit on cancellation.

## Focused verification

The [FPAS xUnit tests](../../../../tests/FpasAmp.Library.Tests/) exercise real
temporary folders, watchers, overlapping sources, pause/clear, permissions/links,
disk restart, schema upgrade, safe checkpoints, rollback and corrupt stores.
Metadata cases cover tags, Unicode multi-values, ordering, rating bounds,
changes, cancellation, lazy/offline images, ambiguous copies, moves and restart.
Playlist cases cover all extensions, parser edge cases, watcher reconciliation,
duplicates/order, URI/path handling, durable lists, late track resolution, schema
upgrades, reset/restart, cancellation and M3U8 round trips.
Album cases cover release/context grouping, cover precedence, shared-folder
ambiguity, sidecar watcher changes, offline reuse, image deduplication and damaged
cache recovery. Native view tests verify virtual realization and slice cleanup.

```sh
dotnet test tests/FpasAmp.Library.Tests/FpasAmp.Library.Tests.fpasproj
```

Run from the repository root after the [local SDK setup](../../../../docs/build/development.md#build-an-fpas-application).
