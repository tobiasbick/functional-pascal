using Fpas.Media.Audio.Decoding;
using Fpas.Media.Audio.Output;
using SoundFlow.Backends.MiniAudio.Devices;
using SoundFlow.Components;

namespace Fpas.Media.Audio.Tests;

/// <summary>Runs the source through a native device callback at two different stream rates.</summary>
public sealed class PlaybackDeviceTests
{
    /// <summary>Window close stops the native device before any shell cleanup or session disposal.</summary>
    [Fact]
    public void ClosingStopsTheNativeStreamBeforeDisposal()
    {
        using var engine = Backend.Open(silent: true);
        using var source = new TrackSource(Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.mp3"));
        using var closing = new CancellationTokenSource();
        using var output = new DeviceStream(engine, source, closing.Token);
        output.Start();
        Assert.True(output.IsRunning);
        closing.Cancel();
        Assert.False(output.IsRunning);
        output.Start();
        Assert.False(output.IsRunning);
    }

    /// <summary>Checks the 300 ms signal change and natural end while switching decoder formats on one engine.</summary>
    [Fact]
    public void RendersAtTheTrackRateAndReleasesTheDevice()
    {
        using var engine = Backend.Open(silent: true);
        foreach (string suffix in new[] { "ogg", "aac.m4a" })
        {
            using var source = new TrackSource(Path.Combine(AppContext.BaseDirectory, "Fixtures", "step." + suffix));
            using var device = engine.InitializePlaybackDevice(null, source.Format, new MiniAudioDeviceConfig());
            using var player = new SoundPlayer(engine, source.Format, source);
            using var ended = new ManualResetEventSlim();
            player.PlaybackEnded += (_, _) => ended.Set();
            int samples = 0, firstTone = -1, firstLoud = -1;
            device.OnAudioProcessed += (buffer, _) =>
            {
                foreach (float value in buffer)
                {
                    if (firstTone < 0 && Math.Abs(value) > .005f) firstTone = samples;
                    if (firstLoud < 0 && Math.Abs(value) > .1f) firstLoud = samples;
                    samples++;
                }
                buffer.Clear(); // The callback measures the signal before sending silence to the device.
            };
            device.MasterMixer.AddComponent(player);
            try
            {
                player.Play();
                device.Start();
                Assert.True(ended.Wait(TimeSpan.FromSeconds(5)), "No playback-end event.");
            }
            finally
            {
                device.Stop();
                device.MasterMixer.RemoveComponent(player);
            }
            Assert.True(firstTone >= 0);
            Assert.InRange((firstLoud - firstTone) / (double)(device.Format.SampleRate * device.Format.Channels), .298, .302);
        }
    }
}
