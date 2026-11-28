// [Documentation](../../../docs/pascal/libraries/media/audio.md#playback)
using Fpas.Media.Audio.Decoding;

namespace Fpas.Media.Audio.Output;

// One decoder and its transition envelope. The callback never takes the command lock.
internal sealed class TrackOutput : IDisposable
{
    private readonly object render = new();
    private readonly IPlaybackSource source;
    private readonly CancellationToken shutdown;
    private readonly int rampFrames;
    private int envelopeFrame;
    private bool retiring;
    private long renderedPosition;
    private bool buffering;
    internal long RequestId;
    private readonly Equalizer equalizer;
    private readonly float[] peaks;
    private readonly int samplesPerSecond;
    private Settings settings;
    private PlaybackState state = PlaybackState.Paused;
    private string? error;

    internal TrackOutput(IPlaybackSource source, string path, Settings settings, CancellationToken shutdown)
    {
        Path = path;
        this.settings = settings;
        this.source = source;
        this.shutdown = shutdown;
        buffering = source.Live;
        samplesPerSecond = checked(source.Format.SampleRate * source.Format.Channels);
        peaks = new float[source.Format.Channels];
        equalizer = new Equalizer(source.Format.SampleRate, peaks.Length, settings);
        rampFrames = Math.Max(1, source.Format.SampleRate / 200); // Five milliseconds.
    }

    internal string Path { get; }
    internal PlaybackState State { get { lock (render) return state; } }
    private TimeSpan Duration => TimeSpan.FromSeconds((double)source.Length / samplesPerSecond);
    internal void Apply(Settings value)
    {
        lock (render)
        {
            settings = value;
            if (state == PlaybackState.Paused) equalizer.Reset(value);
        }
    }

    internal void Play()
    {
        var current = State;
        if (current is PlaybackState.Playing or PlaybackState.Failed) return;
        if (current == PlaybackState.Ended) Seek(TimeSpan.Zero);
        lock (render)
        {
            retiring = false;
            envelopeFrame = 0;
            state = PlaybackState.Playing;
        }
    }

    internal void Pause()
    {
        lock (render)
        {
            if (state == PlaybackState.Playing) state = PlaybackState.Paused;
            Array.Clear(peaks);
        }
    }

    internal void Seek(TimeSpan position)
    {
        if (source.Live) return;
        bool resume = State == PlaybackState.Playing;
        if (resume) FadeOut();
        lock (render)
        {
            if (state == PlaybackState.Failed) return;
            state = PlaybackState.Paused;
        }
        long frames = (long)Math.Round(Math.Clamp(position.TotalSeconds * source.Format.SampleRate, 0,
            source.Length / source.Format.Channels));
        source.Seek((int)frames * source.Format.Channels);
        lock (render)
        {
            Volatile.Write(ref renderedPosition, source.Position);
            equalizer.Reset(settings);
            Array.Clear(peaks);
            retiring = false;
            envelopeFrame = 0;
            state = resume ? PlaybackState.Playing : PlaybackState.Paused;
        }
    }

    internal void SeekFraction(double fraction) => Seek(Duration * Math.Clamp(fraction, 0, 1));

    internal void FadeOut()
    {
        lock (render)
        {
            if (state != PlaybackState.Playing || shutdown.IsCancellationRequested) return;
            retiring = true;
            // Tolerate a disconnected device that never requests the final buffer.
            long deadline = Environment.TickCount64 + 100;
            while (state == PlaybackState.Playing && !shutdown.IsCancellationRequested)
            {
                int remaining = (int)(deadline - Environment.TickCount64);
                if (remaining <= 0) break;
                Monitor.Wait(render, remaining);
            }
        }
    }

    // Available to the close gesture even during slow decoder I/O on the command thread.
    internal PlaybackEvent StoppedSnapshot() => new(Path, PlaybackState.Paused,
        TimeSpan.FromSeconds((double)Volatile.Read(ref renderedPosition) / samplesPerSecond),
        Duration, [], null, Volatile.Read(ref RequestId));

    internal PlaybackEvent Snapshot()
    {
        lock (render)
        {
            var result = new PlaybackEvent(Path, state, TimeSpan.FromSeconds((double)source.Position / samplesPerSecond),
                Duration, [.. peaks], error,
                Buffering: buffering, NowPlaying: source.NowPlaying);
            for (int i = 0; i < peaks.Length; i++) peaks[i] *= 0.75f;
            return result;
        }
    }

    internal void Render(Span<float> output)
    {
        lock (render)
        {
            try
            {
                if (state == PlaybackState.Playing)
                {
                    var target = settings;
                    while (!output.IsEmpty && !shutdown.IsCancellationRequested)
                    {
                        int count = retiring ? Math.Min(output.Length, (envelopeFrame + 1) * peaks.Length) : output.Length;
                        int read = source.ReadBytes(output[..count]);
                        if (read == 0)
                        {
                            Array.Clear(peaks);
                            if (source.Live) buffering = true;
                            else state = PlaybackState.Ended;
                            break;
                        }
                        equalizer.Process(output[..read], target);
                        buffering = false;
                        long framesLeft = source.Live ? long.MaxValue : (source.Length - source.Position + read) / peaks.Length;
                        for (int offset = 0; offset < read; offset += peaks.Length)
                        {
                            float gain = (float)Math.Min(envelopeFrame, framesLeft - 1) / rampFrames;
                            for (int ch = 0; ch < peaks.Length; ch++)
                            {
                                float sample = output[offset + ch] *= gain;
                                peaks[ch] = Math.Max(peaks[ch], Math.Abs(sample));
                            }
                            framesLeft--;
                            envelopeFrame = Math.Clamp(envelopeFrame + (retiring ? -1 : 1), 0, rampFrames);
                        }
                        output = output[read..];
                        if (!shutdown.IsCancellationRequested) Volatile.Write(ref renderedPosition, source.Position);
                        if (retiring && envelopeFrame == 0) { state = PlaybackState.Paused; break; }
                    }
                }
            }
            catch (Exception exception)
            {
                // Never propagate managed decoder failures across the native audio callback.
                error = exception.Message;
                Array.Clear(peaks);
                state = PlaybackState.Failed;
            }
            output.Clear();
            if (retiring) Monitor.PulseAll(render);
        }
    }

    /// <inheritdoc />
    public void Dispose()
    {
        source.Dispose();
    }
}
