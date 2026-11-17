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
    private readonly object lifecycle = new();
    private readonly AudioPlaybackDevice device;
    private readonly CancellationToken shutdown;
    private readonly CancellationTokenRegistration closeRegistration;
    private TrackOutput? voice;

    internal DeviceStream(MiniAudioEngine engine, TrackSource source, CancellationToken shutdown)
    {
        this.shutdown = shutdown;
        device = engine.InitializePlaybackDevice(null, source.Format, new MiniAudioDeviceConfig());
        device.OnAudioProcessed += Render;
        closeRegistration = shutdown.Register(Stop);
    }

    internal bool Accepts(TrackSource source) => device.Format.SampleRate == source.Format.SampleRate
        && device.Format.Channels == source.Format.Channels && device.Format.Format == source.Format.Format;

    internal bool IsRunning => device.IsRunning;

    internal void Bind(TrackOutput? next)
    {
        lock (binding) voice = next;
    }

    internal void Start()
    {
        lock (lifecycle)
        {
            if (!shutdown.IsCancellationRequested && !IsRunning) device.Start();
        }
    }

    private void Stop()
    {
        lock (lifecycle)
        {
            if (IsRunning) device.Stop();
        }
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
        closeRegistration.Dispose();
        try { Stop(); }
        finally
        {
            device.OnAudioProcessed -= Render;
            device.Dispose();
        }
    }
}
