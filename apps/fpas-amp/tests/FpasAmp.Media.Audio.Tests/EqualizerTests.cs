// [Documentation](../../doc/media/audio.md)
using FpasAmp.Media.Audio.Output;

namespace FpasAmp.Media.Audio.Tests;

/// <summary>Measures the ten filters independently at their specified centre frequencies.</summary>
public sealed class EqualizerTests
{
    /// <summary>Rejects a trailing partial frame before modifying the buffer.</summary>
    [Fact]
    public void RejectsPartialInterleavedFrames()
    {
        var eq = new Equalizer(44100, 2, Settings.Flat);
        float[] samples = [2f, 2f, 2f];
        Assert.Throws<InvalidDataException>(() => eq.Process(samples, Settings.Flat));
        Assert.Equal([2f, 2f, 2f], samples);
    }

    /// <summary>Checks the amplitude continuity at a fader change on constant input.</summary>
    [Fact]
    public void VolumeChangesRampAcrossFrames()
    {
        var eq = new Equalizer(44100, 1, Settings.Flat);
        var samples = Enumerable.Repeat(.5f, 1024).ToArray();
        eq.Process(samples, new Settings(-20, new float[10]));
        float previous = .5f;
        foreach (float sample in samples)
        {
            Assert.InRange(Math.Abs(previous - sample), 0, .002f);
            previous = sample;
        }
        Assert.InRange(previous, .0499f, .0501f);
    }

    /// <summary>Checks the response to a live boost or cut against the requested gain in dB.</summary>
    /// <param name="band">Index of the band to adjust.</param>
    /// <param name="frequency">Frequency in Hz of the input sine wave.</param>
    [Theory]
    [InlineData(0, 31)]
    [InlineData(1, 62)]
    [InlineData(2, 125)]
    [InlineData(3, 250)]
    [InlineData(4, 500)]
    [InlineData(5, 1000)]
    [InlineData(6, 2000)]
    [InlineData(7, 4000)]
    [InlineData(8, 8000)]
    [InlineData(9, 16000)]
    public void TracksLiveGainAtBandCentre(int band, int frequency)
    {
        const int rate = 48000;
        var eq = new Equalizer(rate, 1, Settings.Flat);
        foreach (float db in new[] { 9f, -9f, 0f })
        {
            var gains = new float[10];
            gains[band] = db;
            var input = new float[rate];
            for (int i = 0; i < input.Length; i++)
                input[i] = (float)(.1 * Math.Sin(2 * Math.PI * frequency * i / rate));
            eq.Process(input, new Settings(0, gains));
            double sum = 0;
            foreach (float sample in input.AsSpan(rate / 2)) sum += sample * sample;
            double gain = 10 * Math.Log10((sum / (rate / 2)) / .005);
            Assert.InRange(gain, db - .25, db + .25);
        }
    }
}
