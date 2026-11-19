# Fpas.Media.Audio

## Playback

`Session` implements `ISession`. Create one session per player and dispose it
when the player closes. `Open(path)` replaces the current track and prepares it
paused. `Play()` starts or resumes playback and replays an ended track from zero.
`Pause()` holds the current position. `Seek(TimeSpan)` clamps to the track and
preserves play or pause. `Unload()` stops playback, releases the current file
and keeps the session and mixer available for the next `Open(path)`. Seeking after a natural end leaves the track paused.
Tracks with the same PCM format share an output device. The outgoing track
fades to silence and the incoming track fades in over five milliseconds each.
The output supplies silence during decoder seeks and between tracks.
`StopPlayback()` silences the session and returns its final rendered position
without waiting for decoder work. Call it when the window receives a close
request, then dispose the session during cleanup.

Read `Events` from one consumer. Each `PlaybackEvent` carries the file path,
transport state, rendered position, duration and a linear peak per decoded
channel. While playing, status updates arrive roughly 30 times per second.
Transport methods accept an optional caller request ID. Each event carries
the ID of its originating transport action, including subsequent progress or
natural end. A subscriber can discard stale events after selecting the same
file again or seeking.
Unread progress updates are coalesced. The channel also delivers transport
changes, natural ends and file or device failures. A failed session can open
another file. Disposing the session completes the channel.

`Apply(new Settings(volume, gains))` sets volume from −60 to 0 dB and ten
band gains from −12 to +12 dB in the order given by `Settings.Frequencies`.
Settings survive track changes within a session. The device callback smooths
live gain changes and measures the output after equalization, volume and
clipping. Unity gain preserves the original channel levels. Bands at or above
the track's Nyquist frequency are bypassed.

`new Session()` opens the system output on first use (WASAPI on Windows,
PulseAudio or ALSA on Linux). `new Session(true)` uses the real-time null
device. To run the session tests against the system output, set
`FPAS_TEST_SYSTEM_OUTPUT=1`; this plays short test tones.

## Waveform analysis

`Analysis.LoudnessAsync(path, buckets, cancellation)` reads a local track on a
worker thread. It returns a `Task<float[]>` with one RMS value per equal-duration
interval. Channels contribute by energy, including channels in opposite phase.
The loudest interval is 1 and silence is 0. Analysis runs without an audio device.

The library selects the decoder by file extension, case-insensitively.

| Files | Decoder | PCM supplied to the mixer |
| --- | --- | --- |
| `.wav`, `.flac`, `.mp3` | SoundFlow miniaudio | Stereo float at 44.1 kHz |
| `.m4a` (AAC or ALAC), `.ogg` (Vorbis or Opus), `.opus` | SoundFlow FFmpeg | Float at the decoded rate and channel count |

`TrackSource` uses interleaved sample offsets. The session creates the
device with the source's `Format`. This gives the
device backend the correct client rate for conversion to hardware format.
Compressed packet seeks can miss the requested sample. WAV and FLAC use
the native seek. MP3 and the FFmpeg formats decode forward to the target,
reopening the decoder on the same file handle when moving backward. The cost
of a backward seek grows with the target's distance from the start.

Opening Ogg and Opus counts the decoded samples once because the container
duration can include pre-skip. A positive M4A presentation length bounds the
last AAC packet. SoundFlow positions and lengths are signed 32-bit sample counts.

Run `dotnet test tests/Fpas.Media.Audio.Tests/Fpas.Media.Audio.Tests.csproj`
from the repository root. The device test resolves miniaudio's `Null` backend
by name and renders to it on Windows x64 and Linux x64.

## See also

- [Using the media libraries from FPAS](README.md)
- [Audio project](../../../../src/Fpas.Media.Audio/Fpas.Media.Audio.csproj)
