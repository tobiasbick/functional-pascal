# FpasAmp.Imaging

`FpasAmp.Imaging` manages the album thumbnail cache and uses
[FpasAmp.Media.Images](media/images.md) to decode covers.

`ThumbnailStore` accepts encoded pictures on background callers and returns
immutable-by-contract straight-alpha RGBA `Bitmap` values. Pictures are fitted
into a transparent 384 × 384 square. Inputs are bounded to sixteen MiB and
64 megapixels. No music/source file is modified and no URL is fetched.

Encoded image SHA-256 keys share pixel blobs across albums. Per-album references
include the observed audio/sidecar revision; unchanged cached pixels work offline.
Corrupt/missing cache files are ignored and rebuilt. Disk blobs use a 256 MiB
budget with access recency eviction; decoded cache entries use 32 MiB. The GUI
separately retains only its realized viewport. Writes replace temporary files
atomically. Source reads and decoding occur outside the GUI thread.


Construct the store with an absolute, application-owned directory.
`TryGet(album, revision)` returns null if the reference is stale or the pixels
are unavailable. `Store(album, revision, encoded, cancellation)` reuses or
decodes the content blob before publishing the album reference. Concurrent
calls on one store serialize cache access. Cancellation before updating the
album reference can leave a reusable blob for a later call.

The [FPAS thumbnail cache tests](../tests/FpasAmp.Library.Tests/Storage/ThumbnailCache.fpas)
exercise reuse, revision changes and recovery.

## Source files

- [FpasAmp.Imaging.csproj](../src/FpasAmp.Imaging/FpasAmp.Imaging.csproj)
- [ThumbnailStore.cs](../src/FpasAmp.Imaging/ThumbnailStore.cs)
