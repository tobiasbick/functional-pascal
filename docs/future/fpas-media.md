# Media libraries and FPas-Amp playback

The FPas-Amp player strip ([app README](../../apps/fpas-amp/README.md)) holds the
queue, transport, volume and ten-band equalizer in its MVU model, but nothing
produces sound. This plan adds reusable media libraries beside `Fpas.Ui.*` and
then connects real playback to the strip on Windows x64 and Linux x64.

## Libraries

Each library pairs a backend with an FPAS-friendly surface, the same way
`Fpas.Ui.Gui.Cui` wraps CUI. Internally, the libraries may use what FPAS cannot:
span results, pointers, native callbacks and CLR subclassing. Their public APIs
expose immutable records, arrays or `ImmutableArray<T>`, `Task` results,
`ChannelReader<T>` event streams and `IDisposable` sessions. They need no
base classes or span results; FPAS passes arrays to span parameters.

| Library | Language | Backend | Purpose |
| --- | --- | --- | --- |
| `Fpas.Media.Audio` | C# | to be chosen (see below) | Playback session, equalizer, levels and loudness analysis |
| `Fpas.Media.Tags` | FPAS | vendored [ATL](../../third-party/atl/UPSTREAM.md) (`src/Fpas.Atl`) | Tags, embedded pictures, audio properties and playlists as records |
| `Fpas.Media.Images` | C# | PhotoSauce MagicScaler with libjpeg/libpng | Decode, orient and fit pictures into straight-alpha RGBA bitmaps |

- `Fpas.Atl` stays as it is. It deliberately keeps ATL's `ATL` namespace and
  assembly name. `Fpas.Media.Tags` starts from the FPAS code in
  `FpasAmp.Library/Metadata/Internal` and `Playlists/Internal`. It hides ATL's
  global `Settings`, mutable objects and nullable collections. The mapping to
  FPas-Amp's model (sort keys, ratings) stays in the app.
- `Fpas.Media.Images` takes `CoverDecoder` and `Bitmap` from
  `FpasAmp.Imaging`. Its bitmap feeds `Fpas.Ui.Widgets.Media.Images` through
  `Source.Rgba`. `ThumbnailStore`'s album keys, edge size and cache budgets
  remain application logic in FPas-Amp.
- Packaging follows `Fpas.Ui.Gui.Cui`: a package in the local SDK feed when
  consumers outside this repository need it.
- Moving public CLR names requires updating the owning interop and CLR mapping
  specifications in the same change.

### `Fpas.Media.Audio` surface

- `Session`: `IDisposable`. Opens a path and offers play, pause, seek and
  `Apply(Settings)`. `Apply` swaps volume and gains atomically, so moving a
  fader does not click.
- `Settings` record: volume in dB, one gain per equalizer band.
- `Events`: a `ChannelReader<Event>` that publishes status about 30 times a
  second (position, duration, per-channel peak levels, state), plus track end
  and failures. FPas-Amp subscribes to it like the library channels.
- `LoudnessAsync(path, buckets, cancellation)`: a pure analysis that returns
  normalized RMS buckets for the position bar.

Backend candidates, all hidden behind this surface:

- [SoundFlow](https://github.com/LSXPrime/SoundFlow) (MIT, miniaudio
  output) with [SoundFlow.Codecs.FFMpeg](https://www.nuget.org/packages/SoundFlow.Codecs.FFMpeg/)
  for M4A/AAC, Vorbis and Opus. That package is about 39 MB and contains
  FFmpeg/LAME under LGPL 2.1+. Its `ParametricEqualizer` computes
  coefficients only when bands are added or removed, and those changes are not
  synchronized with the audio thread. A C# `SoundModifier` would provide a
  thread-safe ten-band peaking equalizer, and an analyzer would provide the
  levels.
- [Silk.NET.OpenAL](https://www.nuget.org/packages/Silk.NET.OpenAL) (.NET
  Foundation, MIT) with bundled OpenAL Soft (LGPL, separate library). It has a
  buffer-queue model without callbacks. Decoding, equalizer and levels would be
  our own code. A spike confirmed that FPAS can open the default device through
  `AudioContext` and manage sources and buffers; FPAS can now also call the
  `unmanaged`-constrained `BufferData<T>`.

## Steps

- [ ] Choose the audio backend with a C# spike. Test FLAC, MP3, M4A, Ogg Vorbis
  and Opus on Windows and Linux. Check seek accuracy against ATL's duration,
  equalizer changes while dragging, levels, decoding without a device, and a
  null device for headless CI.
- [ ] Create `Fpas.Media.Audio` with the surface above and a fake session for
  tests.
- [ ] Have the shell create and dispose the session next to the library
  session. Turn `Player.Update` into transitions with commands, add progress,
  end and failure messages, and play the next queued track on end.
- [ ] Report unreadable files, unsupported codecs and offline sources in the
  strip and skip them. Stop playback when the track's source is removed,
  reset or cleared.
- [ ] Persist volume and gains in `settings.json` when a drag ends.
- [ ] Analyze loudness for played tracks in the background. Cache the result
  in the library index keyed by the existing SHA-256, and replace the strip's
  uniform bars.
- [ ] Extract `Fpas.Media.Tags` and `Fpas.Media.Images` from FPas-Amp, together
  with the audio library or right after it.
- [ ] Remove the strip's "no audio output" notes from the app README and code
  comments.

## Verification

- Pure tests: player transitions against the fake session, including track
  end at the end of the queue and failures.
- Analysis tests: a generated WAV with known quiet and loud halves produces
  the expected buckets.
- Native GUI tests: the strip follows fake progress and levels. Real device
  output stays a manual check unless the spike finds a null device.

## Open decisions

- Audio backend: SoundFlow with FFmpeg, or OpenAL with our own decoders.
- M4A/AAC support and the size and license of FFmpeg.
- Loudness analysis only for played tracks, or for the whole library in the
  background.
