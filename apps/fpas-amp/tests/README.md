# FPas-Amp tests

Run both FPAS xUnit projects from the FPas-Amp directory after the
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

## Desktop UI

[FpasAmp.Ui.Tests](FpasAmp.Ui.Tests/) checks navigation, catalogue updates,
rating receipts, stale replies and cancellation on a change of view. Source
controls exercise save failures, progress, confirmation and background
acknowledgements. Native checks drive the player strip, queue, transport,
equalizer and compact layout.

Set `FPAS_AMP_UI_ARTIFACTS` to an ignored directory to capture screenshots
of settings, album details and the player.
