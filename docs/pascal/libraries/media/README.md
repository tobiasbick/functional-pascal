# Audio, music tags, playlists and cover images

Reference the media project that owns the operation from your `.fpasproj`.
All three libraries work independently of FPas-Amp and the GUI.

| Operation | Project reference | FPAS import |
| --- | --- | --- |
| Read descriptive tags, audio properties and embedded artwork, or import/export playlists | [Fpas.Media.Tags.fpasproj](../../../../src/Fpas.Media.Tags/Fpas.Media.Tags.fpasproj) | `Fpas.Media.Tags.AudioFiles`, `Fpas.Media.Tags.Pictures`, `Fpas.Media.Tags.Playlists` |
| Decode JPEG or PNG bytes to a square RGBA bitmap | [Fpas.Media.Images.csproj](../../../../src/Fpas.Media.Images/Fpas.Media.Images.csproj) | `Fpas.Media.Images` |
| Play a local file, adjust the mixer or calculate a waveform | [Fpas.Media.Audio.csproj](../../../../src/Fpas.Media.Audio/Fpas.Media.Audio.csproj) | `Fpas.Media.Audio` |

## Inspect a file and its cover

Import `Fpas.Media.Tags.AudioFiles as Tags`, `Fpas.Media.Tags.Pictures as Pictures`
and `Fpas.Media.Images as Images`. The [FPAS consumer tests](../../../../tests/Fpas.Media.Tests/Consumer.fpas)
show the imports and calls in a buildable project.

1. Open a local MP3 with `using const Stream: IO.FileStream := IO.File.OpenRead(Path);`.
   Call `Tags.Read(Stream, '.mp3')` to obtain a detached `Tags.Info`. Performer
   and genre arrays preserve tag order. Missing text is empty and absent numeric
   properties are nullable. The caller retains ownership of the stream.
2. Request embedded artwork separately with `Pictures.Read(Path)`. The result
   is null when the file contains no picture.
3. For a present cover, call `Images.CoverDecoder.Decode(Cover.Data, 192, Token)`
   to obtain an `Images.Bitmap` with a 192-pixel edge.

The decoder applies image orientation and fits the picture into a transparent
square. To display it, pass `Tile.Width`, `Tile.Height` and `Tile.Pixels` to
`Fpas.Ui.Widgets.Media.MediaImages.Source.Rgba`. Keep the bytes unchanged while
the widget uses them. Read tags, artwork and playlists and decode pictures on
a background worker in a GUI.

## Read and write playlists

Import `Fpas.Media.Tags.Playlists as Lists`. `Lists.Read(Path, Token)` returns
an ordered `array of Lists.Item`. Each item carries its original `Location` and
display `Title`. A location can be relative, remote or missing. Reading the
playlist never opens its targets. Resolve relative paths against the playlist
directory when the application needs local files. Use
`Lists.WriteM3u8(Destination, Items, false, Token)` to write a new UTF-8 playlist
at an absolute path. Pass `true` to allow atomic replacement. Resolve
entries to absolute paths before exporting to a different directory to
keep their targets. The writer expresses local paths relative to the
destination when possible.

## Play or analyse a local track

Import `Fpas.Media.Audio as Audio`. Keep an `Audio.Session` alive for the
lifetime of the player. `Open(Path)` prepares a paused track and `Play()`
starts it. Read `Session.Events` from one consumer for position, channel peaks,
natural ends and failures. Use `Pause()` to keep the playhead, `Unload()` to
release the file and `StopPlayback()` when closing the player. Dispose the session
after its workers stop. The [FPas-Amp playback connection](../../../../apps/fpas-amp/src/FpasAmp.Ui/Player/Playback.fpas)
shows the ordered request worker and the event subscription. Optional request IDs identify events from earlier tracks
or seeks.

For a waveform alone, await `Audio.Analysis.LoudnessAsync(Path, Buckets, Token)`.
It runs without an output device and returns one relative RMS value per interval.
Local playback supports WAV, FLAC, MP3, AAC/ALAC in M4A, Ogg Vorbis and Opus on
Windows x64 and Linux x64.

## See also

- [Tag fields, artwork and playlist formats](tags.md)
- [Image formats, limits and native codecs](images.md)
- [Audio session, mixer and waveform reference](audio.md)
- [FPAS consumer tests](../../../../tests/Fpas.Media.Tests/Consumer.fpas)
- [FPas-Amp desktop application](../../../apps/fpas-amp/desktop.md)
- [GUI image widget](../../../ui/gui/text-and-display/image.md)
- [Projects and references](../../program-structure/projects.md)

## Source files

- [Fpas.Media.Tests.fpasproj](../../../../tests/Fpas.Media.Tests/Fpas.Media.Tests.fpasproj)
