# Media libraries and FPas-Amp playback

The FPas-Amp [player strip](../../apps/fpas-amp/README.md) uses the audio
session for transport, equalization and metering on Windows x64 and Linux x64.
This plan tracks the remaining media libraries and playback features.

## Libraries

Each library pairs a backend with an FPAS-friendly surface, the same way
`Fpas.Ui.Gui.Cui` wraps CUI. Internally, the libraries may use what FPAS cannot:
span results, pointers, native callbacks and CLR subclassing. Their public APIs
expose immutable records, arrays or `ImmutableArray<T>`, `Task` results,
`ChannelReader<T>` event streams and `IDisposable` sessions. They need no
base classes or span results; FPAS passes arrays to span parameters.

| Library | Language | Backend | Purpose |
| --- | --- | --- | --- |
| [`Fpas.Media.Audio`](../../src/Fpas.Media.Audio/README.md) | C# | SoundFlow with miniaudio and FFmpeg | Local playback, ten-band equalization, peak metering and waveform analysis |
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

The [audio library reference](../../src/Fpas.Media.Audio/README.md) describes
the session commands, transport events, equalizer settings and waveform API.

The spike compared the following backends and selected SoundFlow.

- [SoundFlow](https://github.com/LSXPrime/SoundFlow) (MIT, miniaudio
  output) with [SoundFlow.Codecs.FFMpeg](https://www.nuget.org/packages/SoundFlow.Codecs.FFMpeg/)
  for M4A/AAC, Vorbis and Opus. That package is about 39 MB and contains
  FFmpeg/LAME under LGPL 2.1+. Its `ParametricEqualizer` computes
  coefficients only when bands are added or removed, and those changes are not
  synchronized with the audio thread. The session applies its own ten peaking
  filters to the decoded output and measures each channel after master gain.
- [Silk.NET.OpenAL](https://www.nuget.org/packages/Silk.NET.OpenAL) (.NET
  Foundation, MIT) with bundled OpenAL Soft (LGPL, separate library). It has a
  buffer-queue model without callbacks. Decoding, equalizer and levels would be
  our own code. A spike confirmed that FPAS can open the default device through
  `AudioContext` and manage sources and buffers; FPAS can now also call the
  `unmanaged`-constrained `BufferData<T>`.

## Spike results

A throwaway C# spike (SoundFlow 1.4.2, SoundFlow.Codecs.FFMpeg 1.4.0) decoded
a generated 10 s tone, 5 s quiet and 5 s loud with different channel levels,
as WAV, FLAC, MP3, AAC and ALAC M4A, Ogg Vorbis and Opus, plus 48 and 96 kHz
variants on Windows and Linux (WSL).

- miniaudio's built-in decoders handle WAV, FLAC and MP3. They convert any
  source rate to the requested 44.1 kHz. The observed seek errors were
  0.1 ms for FLAC/WAV and up to 25 ms for MP3.
- The FFmpeg codec decodes every format with correct levels, but it never
  converts the sample rate: 48 and 96 kHz files and all Opus files would play
  too slowly and too low. Its seeks land on packet boundaries: up to 93 ms off
  for WAV, FLAC and ALAC, 23 ms for AAC and up to 1.2 s for Vorbis and Opus.
  The 44.1 kHz AAC output contained 7.8 ms of extra tail padding. Capping it
  at the container duration preserved the 5 s transition in all 14 spike files.
  Registering FFmpeg globally also takes WAV, FLAC and MP3 away from miniaudio.
  `TrackSource` selects the appropriate factory directly.
- `StreamDataProvider` must be created with an explicit F32 format. Without it,
  16-bit samples are read as floats, which yields NaN levels and noise.
- `SoundPlayer` reports time and duration, seeks, pauses and raises
  `PlaybackEnded`. Its centre pan applies
  -3 dB per channel.
- A C# `SoundModifier` with ten `BiquadFilter` peaking bands (Q 1.41) updates
  coefficients on the audio thread without resetting filter state. An instant
  +12 dB step jumps like a hard volume change; gliding 0.5 dB per buffer cuts
  that step from 3.98 to 1.78 times the steady signal change. An
  `AudioAnalyzer` subclass reads per-channel peaks after volume.
- `AudioEngine.CreateDecoder` decodes without starting a device, at about
  1,000 times real time, which suits loudness analysis.
- SoundFlow 1.4.2 passes managed backend enum numbers directly to miniaudio.
  They differ from the bundled native enum. For example, managed `Null` is 0,
  which selects native WASAPI. `ma_get_backend_from_name` resolves the actual
  IDs. The native `Null` device rendered both 96 kHz Vorbis and 48 kHz AAC
  and raised `PlaybackEnded` on Windows and Linux without a sound server.

### Reproducing the spike

The spike lived in the ignored `.temp-data/audio-spike` directory and may be
gone; this section keeps what is needed to rebuild it.

- Test signal, 44.1 kHz stereo, encoded with ffmpeg to every target format
  (`-c:a flac`, `libmp3lame -b:a 192k`, `aac -b:a 192k`, `alac`,
  `libvorbis -q:a 5`, `libopus -b:a 128k`, plus `-ar 48000`/`96000` variants):
  `aevalsrc=exprs='if(lt(t,5),0.0316*sin(2*PI*220*t),0.5*sin(2*PI*440*t))|if(lt(t,5),0.0158*sin(2*PI*220*t),0.25*sin(2*PI*440*t))':s=44100:d=10`.
  Expected RMS: left quiet 0.0224, left loud 0.3536, right loud 0.1768.
- Seek accuracy: seek to a time before 5.0 s and find the first left sample
  above 0.1; its offset from the expected step is the seek error.
- Equalizer click check: feed a 0.25 amplitude 1 kHz sine in 512-frame
  buffers, step the 1 kHz band to +12 dB and compare the largest
  second difference after the step with the steady value; a pure gain change
  gives about 3.98.
- Linux: run the Windows-built framework-dependent output in WSL with
  `~/.dotnet/dotnet`; the output carries `runtimes/linux-x64/native`
  (`libminiaudio.so`, `libsoundflow-ffmpeg.so`). WSLg provides PulseAudio.


## Steps

- [ ] Skip tracks belonging to offline sources even if the files remain
  accessible. Stop playback when the track's source is removed,
  reset or cleared.
- [ ] Extract `Fpas.Media.Tags` and `Fpas.Media.Images` from FPas-Amp, together
  with the audio library or right after it.
