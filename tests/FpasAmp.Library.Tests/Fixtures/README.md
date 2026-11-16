# Synthetic audio fixtures

The files contain 250 ms of generated stereo silence (MP3/Vorbis at 44.1 kHz,
Opus at 48 kHz). They contain
no third-party recording or personal metadata. Tests copy them into isolated
temporary directories and use ATL to write the tags being exercised.

They were generated with FFmpeg using `anullsrc=r=44100:cl=stereo`, `-t 0.25`,
`-fflags +bitexact -flags:a +bitexact -map_metadata -1 -metadata encoder=` and
either `-c:a libmp3lame -b:a 128k`, `-c:a libvorbis`, or `-c:a libopus -f ogg`.
FFmpeg is not needed to build or run the tests.
