# Fpas.Media.Tags

Add a project reference to [Fpas.Media.Tags.fpasproj](../../../../src/Fpas.Media.Tags/Fpas.Media.Tags.fpasproj).
The library wraps the vendored ATL parser in three FPAS units. Their results are
detached from ATL objects and have no dependency on the music player or SQLite.

| Unit | API | Result |
| --- | --- | --- |
| [AudioFiles](../../../../src/Fpas.Media.Tags/AudioFiles.fpas) | `Read(Stream, Extension)` | `Info` contains credits, genres, release dates, track/disc numbers, sort tags, MusicBrainz IDs, ReplayGain, codec, duration in milliseconds, sample rate in hertz, bitrate in kbit/s, channels and bit depth. |
| [Pictures](../../../../src/Fpas.Media.Tags/Pictures.fpas) | `Read(Path)` | `Cover?` contains the original encoded bytes, MIME type and front-cover flag. A front cover takes precedence over the first picture. An unillustrated file yields null. |
| [Playlists](../../../../src/Fpas.Media.Tags/Playlists.fpas) | `IsExtension`, `IsPlaylist`, `Read`, `WriteM3u8` | Ordered `Item` records retain duplicate entries, location spelling and display titles. |

## Audio properties and descriptive tags

`AudioFiles.Read(Stream, Extension)` requires a readable, seekable stream and
expects an extension including its leading dot. The caller owns the stream.
Reading tags leaves embedded pictures unloaded. ATL must report a positive
duration for the input to be accepted. An unreadable or unseekable stream
raises `ArgumentException`, and a stream without a positive reported duration
raises `InvalidDataException`.

Missing text and lists are empty. Lists split on ATL's display value separator,
trim entries and omit empty values while preserving order. Release dates use
the invariant `yyyy-MM-dd` form when ATL supplies a date. Track and disc counts,
tempo, bitrate, sample rate, channels and bit depth are nullable when unavailable
or nonpositive. `Compilation` is true when ATL exposes either the COMPILATION
or TCMP field with value `1`. ReplayGain fields report file-supplied gains in dB
and linear peaks, as nullable numbers.

Use these `Info` fields when building a library index:

| Index or display need | Fields |
| --- | --- |
| Names and credits | `Title`, `Subtitle`, `Album`, `Artists`, `AlbumArtists`, `Composers` |
| Classification and release | `Genres`, `Grouping`, `Compilation`, `ReleaseDate`, `OriginalReleaseDate` |
| Position in a release | `TrackNumber`, `TrackTotal`, `DiscNumber`, `DiscTotal` |
| Sort keys | `SortTitle`, `SortArtist`, `SortAlbumArtist`, `SortAlbum` |
| MusicBrainz identifiers | `MusicBrainzRecordingId`, `MusicBrainzReleaseId`, `MusicBrainzArtistIds` |
| Other text | `Comment`, `Label`, `Copyright` |
| Audio properties | `Codec`, `DurationMilliseconds`, `Bitrate`, `SampleRate`, `Channels`, `BitDepth`, `Bpm` |
| ReplayGain | `ReplayGainTrack`, `ReplayGainAlbum`, `ReplayPeakTrack`, `ReplayPeakAlbum` |

## Embedded artwork

`Pictures.Read(Path)` selects the first front cover, or the first available
picture when no front cover is marked. It returns null when no picture is
available. An oversized selected picture raises `InvalidDataException`.
`Cover.Data` contains the original encoded bytes, limited to 16 MiB.
`Cover.MimeType` falls back to `application/octet-stream` when ATL omits the value.
Use [Fpas.Media.Images](images.md) to decode JPEG or PNG pictures into pixels.

## Playlist recognition and import

`IsExtension` matches a dot-prefixed extension case-insensitively.
`IsPlaylist(Path)` checks only the path extension. It can return true for
missing and malformed files.

Playlist readers support M3U/M3U8, PLS, XSPF, ASX/WAX/WVX, B4S, DPL,
SMIL/SMI/WPL/ZPL and ATL's experimental FPL reader. The local file limit is
16 MiB. Read operations preserve relative paths and URLs without opening them.
A malformed playlist reports an exception. The cancellation token is checked
around synchronous parsing and while copying entries. Titles use the parser's
display title, prefixed with `Artist - ` when ATL supplies an artist.

## M3U8 export

M3U8 export accepts up to 100,000 entries and requires an absolute `.m3u8`
destination. Relative inputs are interpreted against the destination directory.
Host-local file URIs are converted to paths. Local paths are written relative
to the destination when possible. Non-file URIs and Windows drive or UNC paths
exported on Linux keep their spelling. For an empty title, the export uses the
filename stem. Locations and explicitly supplied titles must be nonblank and
contain no NUL, LF or CR. A relative location starting with `#` receives a
`./` prefix to preserve it as a path. Each entry gets an EXTINF line with
duration `-1`.

The writer publishes UTF-8 without a byte-order mark by moving a temporary file
beside the destination into place. Use `Overwrite` set to `false` to require a new destination, or
`true` to allow replacement.
Cancellation or validation failure before publication preserves an existing
file. Prefer absolute local inputs when exporting a playlist read from a
different folder.

## See also

- [Using the media libraries from FPAS](README.md)
- [FPAS playlist round-trip and atomic replacement tests](../../../../tests/Fpas.Media.Tests/Consumer.fpas)
- [ATL provenance and licensing](../../../../third-party/atl/UPSTREAM.md)

## Source files

- [Fpas.Atl.csproj](../../../../src/Fpas.Atl/Fpas.Atl.csproj)
