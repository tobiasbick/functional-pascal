// [Documentation](../../docs/pascal/libraries/media/audio.md)
using Fpas.Media.Audio.Decoding;
using Fpas.Media.Audio.Output;
using Fpas.Media.Audio.Tests.Support;
using Fpas.Tests.Support;

namespace Fpas.Media.Audio.Tests;

/// <summary>Measures discontinuities at track boundaries and terminal shutdown.</summary>
public sealed class TransitionsTests
{
    /// <summary>Switching between opposite DC levels would click at full amplitude without both ramps.</summary>
    /// <param name="analyze">Also decode the incoming file for its first waveform.</param>
    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task SwitchingTracksRampsThroughSilence(bool analyze)
    {
        using var dir = new TemporaryDirectory();
        string a = dir.File("positive.wav"), b = dir.File("negative.wav");
        Wave.Write(a, 44100 * 4, _ => (16384, 16384));
        Wave.Write(b, 44100 * 4, _ => (-16384, -16384));
        using var first = new TrackOutput(new TrackSource(a), a, Settings.Flat, default);
        using var next = new TrackOutput(new TrackSource(b), b, Settings.Flat, default);
        var buffer = new float[512];
        first.Play();
        first.Render(buffer);
        Assert.InRange(buffer[0], -0.001f, 0.001f);
        Assert.InRange(buffer[^2], 0.49f, 0.51f);
        var analysis = analyze ? Analysis.LoudnessAsync(b, 1024) : Task.FromResult(Array.Empty<float>());
        float previous = buffer[^2];
        var retire = Task.Run(first.FadeOut);
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(3));
        do
        {
            await Task.Delay(1, timeout.Token);
            first.Render(buffer);
            CheckSteps(buffer, ref previous);
        } while (first.State == PlaybackState.Playing);
        await retire;
        Assert.Equal(0f, previous);
        next.Play();
        next.Render(buffer);
        CheckSteps(buffer, ref previous);
        Assert.InRange(previous, -0.51f, -0.49f);
        await analysis;
    }

    private static void CheckSteps(float[] stereo, ref float previous)
    {
        for (int i = 0; i < stereo.Length; i += 2)
        {
            Assert.InRange(Math.Abs(stereo[i] - previous), 0f, 0.004f);
            previous = stereo[i];
        }
    }

    /// <summary>Stop freezes the native playhead before cleanup and pending transport commands cannot restart it.</summary>
    [Fact]
    public async Task ClosingFreezesTheRenderedPositionBeforeDisposal()
    {
        using var session = new Session(silent: true);
        string path = Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.mp3");
        session.Open(path, 1);
        session.Play(2);
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        PlaybackEvent status;
        do { status = await session.Events.ReadAsync(timeout.Token); } while (status.Position == TimeSpan.Zero);
        var final = Assert.IsType<PlaybackEvent>(session.StopPlayback());
        Assert.Equal(2, final.RequestId);
        Assert.Equal(PlaybackState.Paused, final.State);
        Assert.True(final.Position >= status.Position);
        session.Open(path, 3);
        session.Play(4);
        await Task.Delay(100);
        Assert.Equal(final, session.StopPlayback());
        while (session.Events.TryRead(out var update)) Assert.True(update.Position <= final.Position);
    }
}
