# Fpas.Media.Tags

Add a project reference to [Fpas.Media.Tags.fpasproj](Fpas.Media.Tags.fpasproj).
The library wraps the vendored ATL parser in three FPAS units. Their results are
detached from ATL objects and have no dependency on the music player or SQLite.

| Unit | API | Result |
| --- | --- | --- |
| [AudioFiles](AudioFiles.fpas) | `Read(Stream, Extension)` | `Info` contains credits, genres, release dates, track/disc numbers, sort tags, MusicBrainz IDs, ReplayGain, codec, duration in milliseconds, sample rate in hertz, bitrate in kbit/s, channels and bit depth. |
| [Pictures](Pictures.fpas) | `Read(Path)` | `Cover?` contains the original encoded bytes, MIME type and front-cover flag. A front cover takes precedence over the first picture. An unillustrated file yields null. |
| [Playlists](Playlists.fpas) | `IsExtension`, `IsPlaylist`, `Read`, `WriteM3u8` | Ordered `Item` records retain duplicate entries, location spelling and display titles. |

Read audio properties from a readable, seekable stream with the extension including
its dot. The caller owns the stream. Reading tags leaves embedded pictures unloaded.
Missing text and lists are empty. Absent numeric tags are null. Returned cover bytes
are limited to 16 MiB.

Playlist readers support M3U/M3U8, PLS, XSPF, ASX/WAX/WVX, B4S, DPL,
SMIL/SMI/WPL/ZPL and ATL's experimental FPL reader. The local file limit is
16 MiB and the entry limit is 100,000. Read operations preserve relative paths
and URLs without opening them. A malformed playlist reports an exception.
The cancellation token is checked around synchronous parsing and while copying
entries. M3U8 export writes UTF-8 to a temporary file beside the destination,
then moves it into place. Cancellation before publication preserves an existing file.

See the [FPAS media guide](../../docs/pascal/libraries/media.md) for usage and
[ATL provenance](../../third-party/atl/UPSTREAM.md) for backend changes and licensing.
