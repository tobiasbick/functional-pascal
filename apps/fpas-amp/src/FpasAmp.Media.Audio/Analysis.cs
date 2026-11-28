// [Documentation](../../doc/media/audio.md#waveform-analysis)
using FpasAmp.Media.Audio.Decoding;

namespace FpasAmp.Media.Audio;

/// <summary>Offline measurements of local audio files.</summary>
public static class Analysis
{
    /// <summary>Computes the RMS of successive equal-duration intervals across all channels.</summary>
    /// <param name="path">Path of the local audio file.</param>
    /// <param name="buckets">Positive number of intervals spanning the track.</param>
    /// <param name="cancellation">Cancels opening or decoding the file.</param>
    /// <returns>One value per interval, relative to the loudest interval. Silence and empty intervals yield zero.</returns>
    public static Task<float[]> LoudnessAsync(string path, int buckets, CancellationToken cancellation = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        ArgumentOutOfRangeException.ThrowIfNegativeOrZero(buckets);
        return Task.Run(() => Measure(path, buckets, cancellation), cancellation);
    }

    private static float[] Measure(string path, int buckets, CancellationToken cancellation)
    {
        using var source = new TrackSource(path, cancellation);
        int channels = source.Format.Channels;
        int frames = source.Length / channels;
        var buffer = new float[4096 * channels];
        var values = new float[buckets];
        float peak = 0;
        for (int bucket = 0; bucket < buckets; bucket++)
        {
            cancellation.ThrowIfCancellationRequested();
            int end = (int)((bucket + 1L) * frames / buckets) * channels;
            int start = source.Position;
            double squares = 0;
            while (source.Position < end)
            {
                cancellation.ThrowIfCancellationRequested();
                int read = source.ReadBytes(buffer.AsSpan(0, Math.Min(buffer.Length, end - source.Position)));
                if (read == 0) break;
                foreach (float sample in buffer.AsSpan(0, read))
                    squares += (double)sample * sample;
            }
            if (source.Position > start)
            {
                if (!double.IsFinite(squares))
                    throw new InvalidDataException("Audio contains non-finite PCM samples.");
                values[bucket] = (float)Math.Sqrt(squares / (source.Position - start));
                peak = Math.Max(peak, values[bucket]);
            }
        }
        if (peak > 0)
            for (int i = 0; i < values.Length; i++) values[i] /= peak;
        return values;
    }
}
