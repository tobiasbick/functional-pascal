# FPas-Amp tests

Run the FPas-Amp solution from its directory after the
[local SDK setup](../../../docs/build/development.md#build-an-fpas-application):

```sh
dotnet test fpas-amp.slnx
```

On headless Linux, run this command under Xvfb with OpenGL enabled to exercise
the native GUI tests. To select tests that run without a display, use
`dotnet test fpas-amp.slnx --filter 'Category!=Gui'`.

## Library

[FpasAmp.Library.Tests](FpasAmp.Library.Tests/) uses real temporary directories,
watchers and SQLite stores. It checks overlapping sources, pause/resume,
explicit rescans, subtree removal while paused, restart, rollback and recovery.
The permission and symbolic-link cases run as an ordinary Unix user.

Metadata tests use [synthetic MP3, Vorbis and Opus fixtures](FpasAmp.Library.Tests/Fixtures/README.md)
to check Unicode filters, ordering, stable identities after moves, and durable
ratings. Artwork cases check precedence, corruption and offline reuse. Playlist
cases check URI escaping, entry order, duplicates and round trips. See the
[library contract](../doc/library.md#focused-verification) for the focused test command.

## Media libraries

[FpasAmp.Media.Tests](FpasAmp.Media.Tests/) reads MP3 and Vorbis tags and covers,
decodes JPEG and PNG pixels, and checks playlist locations and atomic M3U8
replacement. [FpasAmp.Media.Audio.Tests](FpasAmp.Media.Audio.Tests/) checks codec
seeking, playback events, output devices, mixing and waveforms.
See [audio verification](../doc/media/audio.md#verification) for the individual
test commands.

## Desktop UI

[FpasAmp.Ui.Tests](FpasAmp.Ui.Tests/) checks navigation, catalogue updates,
rating receipts, stale replies and cancellation on a change of view. Source
controls exercise save failures, progress, confirmation and background
acknowledgements. Native checks drive the player strip, queue, transport,
equalizer and compact layout. The track navigation test clicks the title
header in both Music and album track lists and checks that the same file
stays at the cursor and in view as the sort direction changes.

Set `FPAS_AMP_UI_ARTIFACTS` to an absolute path in an ignored directory, then
run the native tests to capture the UI with test data:

```sh
FPAS_AMP_UI_ARTIFACTS="$PWD/../../.temp-data/fpas-amp-screenshots" dotnet test fpas-amp.slnx --filter 'Category=Gui'
```

In PowerShell, set `$env:FPAS_AMP_UI_ARTIFACTS` before running the test command.
The examples in the [user guides](../doc/README.md#user-guides) use these
captures. Use `radio-stations-fixture.png` for the directory screenshot.
