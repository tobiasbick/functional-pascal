# FPas-Amp

FPas-Amp is a desktop music player for Windows and Linux, written in
Functional Pascal.

> [!IMPORTANT]
> **Internet radio requires FFmpeg.** Install FFmpeg and make sure the account
> that launches FPas-Amp can run `ffmpeg -version`. The `ffmpeg` executable
> must be on the app's `PATH`. Local music playback works without it.

## Get started

Set up the [local FPAS SDK](../../docs/build/development.md#build-an-fpas-application)
and the [native CUI build tools](../../docs/ui/gui/native-build.md#prerequisites).
From this directory, run:

```sh
dotnet build fpas-amp.slnx
dotnet run --project src/FpasAmp/FpasAmp.fpasproj --no-build
```

Open **Settings → Library → Add folder** to index your music. Use
**Radio → Stations** to browse online stations or enter a stream URL.

See [Windows launch](doc/desktop.md#windows-desktop-launch) or
[Linux desktop installation](doc/desktop.md#linux-desktop-installation)
to launch from the desktop.

## User guides

| Guide | What it covers |
| --- | --- |
| [Music library and playlists](doc/browsing.md) | Browse, search, filter, rate and open local playlists |
| [Playback](doc/playback.md) | Queue, waveform, volume, equalizer and session restore |
| [Internet radio](doc/radio.md) | Station discovery, favorites and stream URLs |
| [Track details and tag editing](doc/track-metadata.md) | File info, fingerprints and MusicBrainz suggestions |
| [Settings and music sources](doc/settings.md) | Appearance, scan controls and AcoustID keys |

The [documentation index](doc/README.md) also covers user files, internals and
future plans. The [test guide](tests/README.md) lists test and screenshot commands.
