// [Documentation](../../docs/pascal/libraries/media/audio.md)
using Fpas.Tests.Support;
using Fpas.Media.Audio.Tests.Support;

namespace Fpas.Media.Audio.Tests;

/// <summary>Verifies the level units and interval boundaries of the offline waveform.</summary>
public sealed class AnalysisTests
{
    /// <summary>Measures silence, a quarter-amplitude interval and a full-amplitude interval in anti-phase stereo.</summary>
    [Fact]
    public async Task MeasuresChannelEnergyInEqualTimeIntervals()
    {
        using var dir = new TemporaryDirectory();
        string path = dir.File("levels.wav");
        Wave.Write(path, 3 * 4410, frame =>
        {
            int sample = frame < 4410 ? 0 : frame < 8820 ? 4096 : 16384;
            return ((short)sample, (short)-sample);
        });
        var levels = await Analysis.LoudnessAsync(path, 3);
        Assert.Equal(new[] { 0f, .25f, 1f }, levels);
    }
}
