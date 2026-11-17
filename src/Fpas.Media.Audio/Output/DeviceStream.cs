using Fpas.Media.Audio.Decoding;
using SoundFlow.Abstracts.Devices;
using SoundFlow.Backends.MiniAudio;
using SoundFlow.Backends.MiniAudio.Devices;
using SoundFlow.Enums;

namespace Fpas.Media.Audio.Output;

// The native stream belongs to the session. Compatible tracks can replace the
// render source while the device clock keeps running, including periods of silence.
internal sealed class DeviceStream : IDisposable
{
    private readonly object binding = new();
    private readonly AudioPlaybackDevice device;
    private readonly CancellationToken shutdown;
    private TrackOutput? voice;
    private bool started;

    internal DeviceStream(MiniAudioEngine engine, TrackSource source, CancellationToken shutdown)
    {
        this.shutdown = shutdown;
        device = engine.InitializePlaybackDevice(null, source.Format, new MiniAudioDeviceConfig());
        device.OnAudioProcessed += Render;
    }

    internal bool Accepts(TrackSource source) => device.Format.SampleRate == source.Format.SampleRate
        && device.Format.Channels == source.Format.Channels && device.Format.Format == source.Format.Format;

    internal void Bind(TrackOutput? next)
    {
        lock (binding) voice = next;
    }

    internal void Start()
    {
        if (started || shutdown.IsCancellationRequested) return;
        device.Start();
        started = true;
    }

    private void Render(Span<float> samples, Capability capability)
    {
        lock (binding)
        {
            if (voice is null || shutdown.IsCancellationRequested) samples.Clear();
            else voice.Render(samples);
        }
        if (shutdown.IsCancellationRequested) samples.Clear();
    }

    public void Dispose()
    {
        try { if (started) device.Stop(); }
        finally
        {
            device.OnAudioProcessed -= Render;
            device.Dispose();
        }
    }
}
