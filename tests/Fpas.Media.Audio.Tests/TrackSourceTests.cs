// [Documentation](../../docs/pascal/libraries/media/audio.md)
using Fpas.Media.Audio.Decoding;

namespace Fpas.Media.Audio.Tests;

/// <summary>Checks the presentation timeline against the same authored signal in several containers.</summary>
public sealed class TrackSourceTests
{
    /// <summary>Checks duration, the initial signal and the same transition after seeking both ways.</summary>
    /// <param name="suffix">Format of the half-quiet, half-loud fixture.</param>
    /// <param name="toleranceMs">Allowed timing error including lossy coding and filtering.</param>
    [Theory]
    [InlineData("aac.m4a", 2)]
    [InlineData("alac.m4a", 1)]
    [InlineData("ogg", 2)]
    [InlineData("opus", 2)]
    [InlineData("flac", 1)]
    [InlineData("mp3", 3)]
    public void ReadsTheSameTransitionFromAnyPosition(string suffix, int toleranceMs)
    {
        using var source = new TrackSource(Path.Combine(AppContext.BaseDirectory, "Fixtures", "step." + suffix));
        int rate = source.SampleRate, channels = source.Format.Channels;
        Assert.InRange((double)source.Length / channels / rate, 0.6 - toleranceMs / 1000d, 0.6 + toleranceMs / 1000d);
        var chunk = new float[257 * channels];
        foreach (int startMs in new[] { 280, 0, 90 })
        {
            source.Seek(startMs * rate / 1000 * channels);
            int foundFrame = -1;
            while (foundFrame < 0)
            {
                int first = source.Position / channels;
                int read = source.ReadBytes(chunk);
                if (read == 0) break;
                for (int i = 0; i < read; i += channels)
                {
                    if (Math.Abs(chunk[i]) <= 0.15) continue;
                    foundFrame = first + i / channels;
                    break;
                }
            }
            Assert.InRange(foundFrame * 1000d / rate, 300 - toleranceMs, 300 + toleranceMs);
        }
        source.Seek(source.Length);
        Assert.Equal(0, source.ReadBytes(chunk));
    }
}
