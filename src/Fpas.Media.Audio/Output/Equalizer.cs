// [Documentation](../../../docs/pascal/libraries/media/audio.md#mixer-settings)
using SoundFlow.Enums;
using SoundFlow.Utils;

namespace Fpas.Media.Audio.Output;

internal sealed class Equalizer
{
    private readonly int rate;
    private readonly int channels;
    private readonly BiquadFilter[,] filters;
    private readonly float[] gains = new float[Settings.Frequencies.Length];
    private readonly int[] active = new int[Settings.Frequencies.Length];
    private float volume;

    internal Equalizer(int rate, int channels, Settings settings)
    {
        this.rate = rate;
        this.channels = channels;
        filters = new BiquadFilter[gains.Length, channels];
        for (int band = 0; band < gains.Length; band++)
            for (int channel = 0; channel < channels; channel++)
                filters[band, channel] = new BiquadFilter();
        Reset(settings);
    }

    internal void Reset(Settings settings)
    {
        for (int band = 0; band < gains.Length; band++)
        {
            for (int channel = 0; channel < channels; channel++) filters[band, channel].Reset();
            SetGain(band, settings.Gains[band]);
        }
        volume = Linear(settings.Volume);
    }

    internal void Process(Span<float> buffer, Settings settings)
    {
        if (buffer.Length % channels != 0)
            throw new InvalidDataException("Audio samples do not contain complete frames.");
        float target = Linear(settings.Volume);
        while (!buffer.IsEmpty)
        {
            int frames = Math.Min(128, buffer.Length / channels);
            int count = 0;
            for (int band = 0; band < gains.Length; band++)
            {
                if (Settings.Frequencies[band] * 2 >= rate) continue;
                float difference = settings.Gains[band] - gains[band];
                if (difference != 0) SetGain(band, gains[band] + Math.Clamp(difference, -0.5f, 0.5f));
                if (gains[band] != 0 || difference != 0) active[count++] = band;
            }
            // Ramp master gain across individual frames, at most one full-scale change in 10 ms.
            float step = Math.Clamp((target - volume) / frames, -100f / rate, 100f / rate);
            for (int frame = 0; frame < frames; frame++)
            {
                volume += step;
                for (int channel = 0; channel < channels; channel++)
                {
                    ref float sample = ref buffer[frame * channels + channel];
                    for (int i = 0; i < count; i++) sample = filters[active[i], channel].Process(sample);
                    if (!float.IsFinite(sample)) throw new InvalidDataException("Non-finite audio sample.");
                    sample = Math.Clamp(sample * volume, -1f, 1f);
                }
            }
            if (Math.Abs(target - volume) < 0.000001f) volume = target;
            buffer = buffer[(frames * channels)..];
        }
    }

    private void SetGain(int band, float value)
    {
        gains[band] = value;
        int frequency = Settings.Frequencies[band];
        if (frequency * 2 >= rate) return;
        for (int channel = 0; channel < channels; channel++)
            filters[band, channel].Update(FilterType.Peaking, rate, frequency, 1.41f, value, 1f);
    }

    private static float Linear(float db) => MathF.Pow(10, db / 20);
}
