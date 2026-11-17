using Fpas.Media.Audio.Tests.Support;
using Fpas.Tests.Support;

namespace Fpas.Media.Audio.Tests;

/// <summary>Verifies transport events and metering through the native audio callback.</summary>
public sealed class SessionTests
{
    // The same transport test can exercise the desktop's default output during manual acceptance.
    private static Session OpenSession() => new(silent: Environment.GetEnvironmentVariable("FPAS_TEST_SYSTEM_OUTPUT") != "1");

    /// <summary>A bad decoded sample reports a failed track across the native callback boundary.</summary>
    [Fact]
    public async Task ReportsInvalidPcmAndOpensTheNextTrack()
    {
        using var dir = new TemporaryDirectory();
        string path = dir.File("invalid.wav");
        using (var writer = new BinaryWriter(File.Create(path)))
        {
            // IEEE float stereo WAVE, containing one non-finite audio frame.
            writer.Write("RIFF"u8);
            writer.Write(44);
            writer.Write("WAVEfmt "u8);
            writer.Write(16);
            writer.Write((short)3);
            writer.Write((short)2);
            writer.Write(44100);
            writer.Write(44100 * 8);
            writer.Write((short)8);
            writer.Write((short)32);
            writer.Write("data"u8);
            writer.Write(8);
            writer.Write(float.NaN);
            writer.Write(0f);
        }
        using var session = new Session(true);
        session.Open(path);
        await Until(session, e => e.State == PlaybackState.Paused);
        session.Play();
        var failure = await Until(session, e => e.State == PlaybackState.Failed);
        Assert.Equal(path, failure.Path);
        Assert.False(string.IsNullOrWhiteSpace(failure.Error));
        session.Open(Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.mp3"));
        session.Play();
        await Until(session, e => e.State == PlaybackState.Playing && e.Position > TimeSpan.Zero);
    }

    /// <summary>Checks pause, live and paused seeks, replay, opening after failure and stream completion.</summary>
    [Fact]
    public async Task ControlsRenderingAndRecoversFromAFailedOpen()
    {
        using var session = OpenSession();
        string path = Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.alac.m4a");
        session.Open(path + ".missing");
        var failure = await Until(session, e => e.State == PlaybackState.Failed);
        Assert.False(string.IsNullOrWhiteSpace(failure.Error));
        session.Apply(new Settings(-48, new float[10]));
        session.Open(path);
        var ready = await Until(session, e => e.State == PlaybackState.Paused);
        Assert.Equal(path, ready.Path);
        Assert.Equal(TimeSpan.FromMilliseconds(600), ready.Duration);
        Assert.Equal(TimeSpan.Zero, ready.Position);
        session.Play();
        await Until(session, e => e.State == PlaybackState.Playing && e.Position > TimeSpan.Zero);
        session.Pause();
        var paused = await Until(session, e => e.State == PlaybackState.Paused);
        Assert.All(paused.Peaks, p => Assert.Equal(0, p));
        await Task.Delay(100);
        session.Pause();
        Assert.Equal(paused.Position, (await Until(session, e => e.State == PlaybackState.Paused)).Position);
        session.Seek(TimeSpan.FromMilliseconds(150));
        Assert.Equal(TimeSpan.FromMilliseconds(150), (await Until(session, e => e.State == PlaybackState.Paused)).Position);
        session.Play();
        session.Seek(TimeSpan.FromMilliseconds(500));
        await Until(session, e => e.State == PlaybackState.Playing && e.Position.TotalMilliseconds >= 500);
        var end = await Until(session, e => e.State == PlaybackState.Ended);
        Assert.Equal(end.Duration, end.Position);
        session.Play();
        await Until(session, e => e.State == PlaybackState.Playing && e.Position < TimeSpan.FromMilliseconds(300));
        var next = Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.opus");
        session.Open(next);
        var replaced = await Until(session, e => e.Path == next && e.State == PlaybackState.Paused);
        Assert.Equal(TimeSpan.Zero, replaced.Position);
        session.Play();
        await Until(session, e => e.Path == next && e.State == PlaybackState.Playing && e.Position > TimeSpan.Zero);
        session.Dispose();
        while (session.Events.TryRead(out _)) { }
        await session.Events.Completion.WaitAsync(TimeSpan.FromSeconds(2));
    }

    /// <summary>Checks live volume and 500 Hz EQ changes against known independent stereo amplitudes.</summary>
    [Fact]
    public async Task ReportsPostFaderPeaksForEachChannel()
    {
        using var dir = new TemporaryDirectory();
        string path = dir.File("stereo.wav");
        Wave.Write(path, 44100 * 5, i =>
        {
            double wave = Math.Sin(2 * Math.PI * 500 * i / 44100);
            return ((short)(3277 * wave), (short)(819 * wave));
        });
        using var session = OpenSession();
        session.Open(path);
        session.Play();
        await Peaks(.100f, .025f);
        var gains = new float[10];
        session.Apply(new Settings(-12, gains));
        await Peaks(.0251f, .00628f);
        gains[4] = 6;
        session.Apply(new Settings(-12, gains));
        gains[4] = -12;
        await Peaks(.0501f, .01253f);
        session.Dispose();
        using var exclusive = File.Open(path, FileMode.Open, FileAccess.ReadWrite, FileShare.None);

        async Task Peaks(float left, float right)
        {
            await Until(session, e => e.State == PlaybackState.Playing && e.Peaks.Length == 2 &&
                Math.Abs(e.Peaks[0] - left) < left * .05f && Math.Abs(e.Peaks[1] - right) < right * .05f);
        }
    }

    private static async Task<PlaybackEvent> Until(ISession session, Func<PlaybackEvent, bool> matches)
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        await foreach (var update in session.Events.ReadAllAsync(timeout.Token))
        {
            if (matches(update)) return update;
            Assert.True(update.State != PlaybackState.Failed, update.Error);
        }
        throw new InvalidOperationException("Session closed before the expected event.");
    }
}
