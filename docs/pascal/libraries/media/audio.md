# Fpas.Media.Audio

## Playback

`Session` implements `ISession`. Create one session per player and dispose it
when the player closes. `Open(path)` replaces the current track and prepares it
paused. `Play()` starts or resumes playback and replays an ended track from zero.
`Pause()` holds the current position. `Seek(TimeSpan)` clamps to the track and
preserves play or pause. `Unload()` stops playback, releases the current file
and keeps the session and mixer available for the next `Open(path)`. Seeking
after a natural end leaves the track paused. `Unload()` does not publish a
transport event.

`Open` and `Seek` can do blocking decoder work. In a GUI, send transport
requests to an ordered background worker. The [FPas-Amp playback
connection](../../../../apps/fpas-amp/src/FpasAmp.Ui/Player/Playback.fpas)
shows a request mailbox and a separate event consumer.
Tracks with the same PCM format share an output device. The outgoing track
fades to silence and the incoming track fades in over five milliseconds each.
The output supplies silence during decoder seeks and between tracks.
`StopPlayback()` silences the session and returns its final rendered position
without waiting for decoder work, as a `PlaybackEvent` snapshot or null when
no track is open. It permanently shuts down transport and repeat calls return
the same snapshot. Call it when the window receives a close request, then
dispose the session during cleanup.

Read `Events` from one consumer. Each `PlaybackEvent` carries the file path,
transport state, rendered position, duration and a linear peak per decoded
channel. While playing, status updates arrive roughly 30 times per second.
Transport methods accept an optional caller request ID. Each event carries
the ID of its originating transport action, including subsequent progress or
natural end. A subscriber can discard stale events after selecting the same
file again or seeking.
Periodic progress is skipped while an earlier event remains unread.
`PlaybackEvent.State` is `Paused`, `Playing`, `Ended` or `Failed`.
`Error` carries a message on failure. The channel also delivers transport
changes, natural ends and file or device failures. A failed session can open
another file. Disposing the session completes the channel.

## Mixer settings

`Apply(new Settings(volume, gains))` sets volume from −60 to 0 dB and ten
band gains from −12 to +12 dB. The constructor copies the gains and requires
finite values. `Settings.Frequencies` orders them from 31 Hz to 16 kHz
(31, 62, 125, 250, 500, 1000, 2000, 4000, 8000 and 16000).
`Settings.Flat` starts at 0 dB volume and 0 dB across all bands.
Settings survive track changes within a session. The device callback smooths
live gain changes and measures the output after equalization, volume and
clipping. Unity gain preserves the original channel levels. Bands at or above
the track's Nyquist frequency are bypassed.

## Output devices

`new Session()` opens the system output on first use (WASAPI on Windows,
PulseAudio or ALSA on Linux). `new Session(true)` uses the real-time null
device. To run the session tests against the system output, set
`FPAS_TEST_SYSTEM_OUTPUT=1`; this plays short test tones.

## Waveform analysis

`Analysis.LoudnessAsync(path, buckets, cancellation)` reads a local track on a
worker thread. It returns a `Task<float[]>` with one RMS value per equal-duration
interval. Channels contribute by energy, including channels in opposite phase.
The loudest nonsilent interval is 1. Silence and empty intervals are 0.
Use a positive bucket count and await the task to observe decoding failures
or cancellation. Analysis runs without an audio device and uses the same
decoder and channel layout as playback.

## Local formats and seek behavior

The library selects the decoder by file extension, case-insensitively.

| Files | Decoder | PCM supplied to the mixer |
| --- | --- | --- |
| `.wav`, `.flac`, `.mp3` | SoundFlow miniaudio | Stereo float at 44.1 kHz |
| `.m4a` (AAC or ALAC), `.ogg` (Vorbis or Opus), `.opus` | SoundFlow FFmpeg | Float at the decoded rate and channel count |

`TrackSource` uses interleaved sample offsets. The session creates the
device with the source's `Format`. This gives the
device backend the correct client rate for conversion to hardware format.
Compressed packet seeks can miss the requested sample. WAV and FLAC try
the native seek first. MP3 and the FFmpeg formats decode forward to the target,
reopening the decoder on the same file handle when moving backward. The cost
of a backward seek grows with the target's distance from the start.

Opening Ogg and Opus counts the decoded samples once because the container
duration can include pre-skip. An unavailable decoder length also triggers
this count. A positive M4A presentation length bounds the
last AAC packet. SoundFlow positions and lengths are signed 32-bit sample counts.

## Verification

Run `dotnet test tests/Fpas.Media.Audio.Tests/Fpas.Media.Audio.Tests.csproj`
from the repository root. The device test resolves miniaudio's `Null` backend
by name and renders to it on Windows x64 and Linux x64.

Run `dotnet test tests/Fpas.Media.Tests/Fpas.Media.Tests.fpasproj` with the
[local FPAS SDK](../../../build/development.md#build-an-fpas-application) to
verify the public analysis call from FPAS.

## See also

- [Using the media libraries from FPAS](README.md)
- [Audio project](../../../../src/Fpas.Media.Audio/Fpas.Media.Audio.csproj)

## Source files

<details>
<summary>Browse 18 source files</summary>

- `src/Fpas.Media.Audio/`
  - [Analysis.cs](../../../../src/Fpas.Media.Audio/Analysis.cs)
  - [Playback.cs](../../../../src/Fpas.Media.Audio/Playback.cs)
  - [Session.cs](../../../../src/Fpas.Media.Audio/Session.cs)
  - [Settings.cs](../../../../src/Fpas.Media.Audio/Settings.cs)
- `src/Fpas.Media.Audio/Decoding/`
  - [TrackSource.cs](../../../../src/Fpas.Media.Audio/Decoding/TrackSource.cs)
- `src/Fpas.Media.Audio/Output/`
  - [Backend.cs](../../../../src/Fpas.Media.Audio/Output/Backend.cs)
  - [DeviceStream.cs](../../../../src/Fpas.Media.Audio/Output/DeviceStream.cs)
  - [Equalizer.cs](../../../../src/Fpas.Media.Audio/Output/Equalizer.cs)
  - [TrackOutput.cs](../../../../src/Fpas.Media.Audio/Output/TrackOutput.cs)
- `tests/Fpas.Media.Audio.Tests/`
  - [AnalysisTests.cs](../../../../tests/Fpas.Media.Audio.Tests/AnalysisTests.cs)
  - [EqualizerTests.cs](../../../../tests/Fpas.Media.Audio.Tests/EqualizerTests.cs)
  - [Fpas.Media.Audio.Tests.csproj](../../../../tests/Fpas.Media.Audio.Tests/Fpas.Media.Audio.Tests.csproj)
  - [PlaybackDeviceTests.cs](../../../../tests/Fpas.Media.Audio.Tests/PlaybackDeviceTests.cs)
  - [SessionTests.cs](../../../../tests/Fpas.Media.Audio.Tests/SessionTests.cs)
  - [TrackSourceTests.cs](../../../../tests/Fpas.Media.Audio.Tests/TrackSourceTests.cs)
  - [TransitionsTests.cs](../../../../tests/Fpas.Media.Audio.Tests/TransitionsTests.cs)
- `tests/Fpas.Media.Audio.Tests/Support/`
  - [FakeSession.cs](../../../../tests/Fpas.Media.Audio.Tests/Support/FakeSession.cs)
  - [Wave.cs](../../../../tests/Fpas.Media.Audio.Tests/Support/Wave.cs)

</details>
