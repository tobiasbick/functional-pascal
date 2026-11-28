# Decoder fixtures

The six files encode a generated 600 ms mono 500 Hz tone. Its amplitude rises
from 0.04 to 0.5 at 300 ms. No recorded music is used. To regenerate one file,
substitute its sample rate, encoder and filename in the command below.

```sh
ffmpeg -f lavfi -i 'aevalsrc=if(lt(t\,0.3)\,0.04\,0.5)*sin(2*PI*500*t):s=RATE:d=0.6' \
    -map_metadata -1 -fflags +bitexact -flags:a +bitexact -c:a ENCODER FILENAME
```

| File | Rate (Hz) | Encoder |
| --- | ---: | --- |
| step.aac.m4a | 48000 | aac |
| step.alac.m4a | 44100 | alac |
| step.ogg | 96000 | libvorbis |
| step.opus | 48000 | libopus |
| step.flac | 48000 | flac |
| step.mp3 | 44100 | libmp3lame |
