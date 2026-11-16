# FpasAmp.Library

UI-independent FPAS library for music locations and presence-only discovery.
It uses the .NET framework, FPAS record support and Microsoft's `Microsoft.Data.Sqlite`
provider, with no GUI or MVU dependency. No media file is opened, decoded,
fingerprinted or inspected for tags, ratings or other music metadata.

## Lifetime and results

Call `Inventory.Monitoring.Session.Start` with normalized-compatible
`Sources.MusicSource` settings, an absolute local SQLite database path and a parent
cancellation token. `Persistence.Paths.DatabasePath` resolves the app's default.
Start validates
and copies settings, then schedules one worker on the thread pool. Directory
enumeration, database opening/loading/writing and watcher installation happen
exclusively on that worker. `:memory:` selects an isolated SQLite database for tests.

Read `Session.Changes` asynchronously. Its capacity is one: a slow consumer gets
the latest completed snapshot instead of accumulating every intermediate result.
Snapshots contain a revision, configuration generation, per-source availability,
immutable path collections, unreadable directory reports and watcher coverage.
Paths describe filesystem entries; matching an extension does not establish
that a file contains valid or complete audio. Snapshots do not compare file contents.
Startup data from SQLite has `Snapshot.Cached = true` and active sources have
`Availability.Cached` until checked. A fresh reconciliation supersedes cached data
in the latest-value stream; a reader may receive the fresh result directly.

`Configure` replaces the settings without waiting for a scan. Pending
configurations collapse to the latest one; results from replaced runs are not
published. Names and equivalent extension sets do not cause rescans. `Refresh`
requests a complete check; `Refresh(SourceId)` targets one online, unpaused source.
`MusicSource.Paused` retains its source-owned index and last progress while other
sources continue. Resuming schedules a full reconciliation, conservatively
covering every change while paused without storing an unbounded event log.
An obsolete scan's in-flight directory/transaction is completed or rolled back
cleanly; unfinished scopes still needed by active sources are requeued.
Await `DisposeAsync` on every exit, normally with
`await using const`; it cancels and joins the worker and releases watchers.
`Completion` exposes worker failures. Drain any remaining snapshot before
awaiting the channel reader's own completion.

Read `Session.Progress` separately for small, latest-value notifications, normally
at most ten per second plus immediate state transitions. It reports queued,
scanning, idle, paused and offline phases, real files found/directories checked,
and completed availability/membership counts. Progress does not copy full path
arrays or assume a known total.

`ClearAsync(SourceId, Cancellation)` pauses and clears one source; an empty ID
clears all sources. The returned snapshot acknowledges a committed transaction,
not merely queue admission. The UI persists the pause preferences before
requesting the operation. Duplicate pending targets share one acknowledgement;
at most sixteen distinct clear operations queue. Caller cancellation stops
waiting while an accepted operation remains owned by the library lifetime.
Clearing retains another source's shared rows and never deletes media/preferences.

## Reconciliation

- Enumerate one directory at a time with `DirectoryInfo.EnumerateFileSystemInfos`.
  Include ordinary hidden directories. Skip reparse points and reject roots
  reached through directory links or junctions. Paths compare ordinally on Unix
  and without case on Windows; symbolic aliases and hard links are not merged.
- Merge equal or nested active roots for physical traversal and watchers. Each
  configured source retains its own ID and extension-filtered view, so overlapping
  sources can refer to the same physical entry without duplicate discovery work.
  Sources with no selected formats schedule neither traversal nor a watcher.
- Enable recursive `FileSystemWatcher` instances before the first full scan.
  Callbacks only mark directories. File writes recheck their direct parent;
  structural events recheck its subtree. New directories are always traversed.
- Treat events as invalidations rather than a reliable operation log. Enumerate
  the affected directories and reconcile the actual membership. Overflow/error
  or a saturated dirty set requests a complete check and watcher reinstallation.
- Memberships and errors are source-scoped even though traversal is shared.
  A paused/cleared source cannot inherit another source's background updates.
  Failed enumeration retains known entries and descendants. A missing root is
  unavailable, rather than evidence that its files were removed. Offline sources
  retain their last completed view and schedule no work of their own. Complete
  checks retry missing locations and unavailable watcher installations.

## Local index

`library.db` belongs in the per-user `LocalApplicationData/fpas-amp` directory,
separately from music roots and the JSON preferences. `settings.json` owns source
configuration; SQLite stores only the IDs/root context needed to associate observations.

- `files`: one normalized absolute path, directory key and stable integer ID.
  An unchanged path retains its ID across scans/restarts. A rename is a different
  path; content identity and metadata-preserving rename tracking are outside this index.
- `source_files`: many-to-many source membership, sharing a file row across
  overlapping sources. Paused, offline and unavailable memberships survive other
  sources' successful reconciliation.
- `directories`: per-source directory context, last successful UTC check and
  latest error. Discovered but unchecked child directories have no success time.

The single worker uses parameterized SQL, prepared row commands, WAL, foreign keys
and full SQLite durability. Each directory observation has one transaction. Only
complete enumeration removes missing files/child subtrees. Partial observations
can add positively observed paths and report errors, but retain unknown entries
and their last successful check. Cancellation or an obsolete generation rolls back
the current transaction; earlier complete checkpoints remain usable after restart.
Unused file rows are collected after their last source association is removed.

Source removal, root changes and excluded formats update the index as configuration
changes; they do not imply filesystem deletion. SQLite failures fault `Completion`
and the snapshot reader. Corrupt or unsupported databases are reported and left
intact, rather than silently replaced. The database remains a reconstructible
presence index; no application ratings or other irreplaceable user data belong here.

Database and WAL/SHM/journal notifications are ignored by the music watchers so
writing the index cannot continually trigger new scans. Runtime database files
and their sidecars are ignored by Git. Native SQLite assets arrive through the
provider's NuGet dependencies.

Default limits: one worker, a fixed **500 ms** notification batching window,
**1,024** dirty directories and a complete control scan **15 minutes** after
the last full check. `Inventory.Model.Options` configures these values. Continuous
notifications cannot extend the batching window indefinitely. Pending work is
bounded; the index and completed snapshots necessarily scale with discovered
directories and matching paths. There is no task per file.

Cancellation is cooperative between entries. A blocking OS or network filesystem
call cannot be forcibly interrupted by a .NET cancellation token, so cleanup may
wait for that call while the UI thread remains free.

The application starts this lifetime after loading settings, reconfigures it
after successful saves and awaits cleanup after its window closes. Presentation
of the inventory is independent of discovery and is not part of this library.

## Focused verification

The [FPAS xUnit tests](../../../../tests/FpasAmp.Library.Tests/) exercise real
temporary directories, overlapping sources, filtering, locked media, structural
notifications, missing roots, Unix permissions and links, control scans, bounded
invalidations, configuration changes, pause/resume, targeted rescans, clear
acknowledgements and asynchronous shutdown. Storage tests cover
real database restart, cached/offline/unavailable views, stable/shared IDs, Unicode
paths, completed/failed checkpoints, rollback, orphan collection and preserved errors.

```sh
dotnet test tests/FpasAmp.Library.Tests/FpasAmp.Library.Tests.fpasproj
```

Run from the repository root after the [local SDK setup](../../../../docs/build/development.md#build-an-fpas-application).
