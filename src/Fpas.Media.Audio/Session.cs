// [Documentation](../../docs/pascal/libraries/media/audio.md#playback)
using System.Threading.Channels;
using Fpas.Media.Audio.Output;
using Fpas.Media.Audio.Decoding;
using SoundFlow.Backends.MiniAudio;

namespace Fpas.Media.Audio;

/// <summary>Owns local file playback, the output device and the transport event stream.</summary>
public sealed class Session : ISession
{
    private readonly object commands = new();
    private readonly Channel<PlaybackEvent> events = Channel.CreateUnbounded<PlaybackEvent>();
    private readonly Timer timer;
    private readonly bool silent;
    private MiniAudioEngine? engine;
    private DeviceStream? stream;
    private readonly CancellationTokenSource shutdown = new();
    private readonly object closing = new();
    private PlaybackEvent? finalPosition;
    private TrackOutput? track;
    private Settings settings = Settings.Flat;
    private PlaybackState lastState;
    private long lastPublishedId;
    private long requestId;
    private PlaybackEvent? failure;
    private bool disposed;

    /// <summary>Creates a session using the system output device on first open.</summary>
    public Session() : this(false) { }

    /// <summary>Creates a session with real-time rendering and optional silent output.</summary>
    /// <param name="silent">Use miniaudio's null device for offline transport or headless hosts.</param>
    public Session(bool silent)
    {
        this.silent = silent;
        timer = new Timer(_ => Tick(), null, Timeout.Infinite, Timeout.Infinite);
    }

    /// <inheritdoc />
    public ChannelReader<PlaybackEvent> Events => events.Reader;

    /// <inheritdoc />
    public void Open(string path, long requestId = 0)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        OpenSource(path, requestId, () => new TrackSource(path, shutdown.Token));
    }

    /// <inheritdoc />
    public void OpenStream(Uri address, long requestId = 0)
    {
        ArgumentNullException.ThrowIfNull(address);
        if (!address.IsAbsoluteUri || address.Scheme is not ("http" or "https") || address.UserInfo.Length != 0)
            throw new ArgumentException("Radio requires an HTTP or HTTPS address.", nameof(address));
        OpenSource(address.AbsoluteUri, requestId, () => new LiveSource(address, shutdown.Token));
    }

    private void OpenSource(string path, long requestId, Func<IPlaybackSource> create)
    {
        lock (commands)
        {
            ObjectDisposedException.ThrowIf(disposed, this);
            if (shutdown.IsCancellationRequested) return;
            try
            {
                if (track is { } previous)
                {
                    previous.FadeOut();
                    PublishTerminal(previous);
                    CloseTrack();
                }
                this.requestId = requestId;
                failure = null;
                engine ??= Backend.Open(silent);
                var source = create();
                try
                {
                    shutdown.Token.ThrowIfCancellationRequested();
                    if (stream is null || !stream.Accepts(source))
                    {
                        stream?.Dispose();
                        stream = null;
                        stream = new DeviceStream(engine, source, shutdown.Token);
                    }
                    var next = new TrackOutput(source, path, settings, shutdown.Token) { RequestId = requestId };
                    Volatile.Write(ref track, next);
                    stream.Bind(next);
                }
                catch { source.Dispose(); throw; }
                Publish(track);
            }
            catch (Exception error)
            {
                this.requestId = requestId;
                Fail(path, error);
            }
        }
    }

    /// <inheritdoc />
    public void Play(long requestId = 0) => Command(requestId, output =>
    {
        output.Play();
        stream!.Start();
    });

    /// <inheritdoc />
    public void Pause(long requestId = 0) => Command(requestId, output => output.Pause());

    /// <inheritdoc />
    public void Seek(TimeSpan position, long requestId = 0) => Command(requestId, output => output.Seek(position));

    /// <inheritdoc />
    public void SeekFraction(double fraction, long requestId = 0)
    {
        if (!double.IsFinite(fraction)) throw new ArgumentOutOfRangeException(nameof(fraction));
        Command(requestId, output => output.SeekFraction(fraction));
    }

    /// <inheritdoc />
    public void Unload()
    {
        lock (commands)
        {
            ObjectDisposedException.ThrowIf(disposed, this);
            CloseTrack();
            failure = null;
        }
    }

    /// <inheritdoc />
    public PlaybackEvent? StopPlayback()
    {
        // Independent of the command lock: opening and seeking a compressed file can take seconds.
        lock (closing)
        {
            if (!shutdown.IsCancellationRequested)
            {
                shutdown.Cancel();
                finalPosition = Volatile.Read(ref track)?.StoppedSnapshot();
            }
            return finalPosition;
        }
    }

    /// <inheritdoc />
    public void Apply(Settings settings)
    {
        ArgumentNullException.ThrowIfNull(settings);
        lock (commands)
        {
            ObjectDisposedException.ThrowIf(disposed, this);
            if (shutdown.IsCancellationRequested) return;
            this.settings = settings;
            track?.Apply(settings);
        }
    }

    private void Command(long id, Action<TrackOutput> action)
    {
        lock (commands)
        {
            ObjectDisposedException.ThrowIf(disposed, this);
            if (shutdown.IsCancellationRequested) return;
            try
            {
                if (track is { } active) PublishTerminal(active);
                if (failure is { } failed && id != requestId)
                    events.Writer.TryWrite(failed with { RequestId = id });
                requestId = id;
                if (track is not { } current || failure is not null) return;
                action(current);
                Volatile.Write(ref current.RequestId, id);
                Publish(current);
            }
            catch (Exception error)
            {
                requestId = id;
                Fail(track!.Path, error);
            }
        }
    }

    private void Tick()
    {
        lock (commands)
        {
            if (disposed || shutdown.IsCancellationRequested || track is not { } current || lastState != PlaybackState.Playing) return;
            try
            {
                // Keep at most one unread periodic update. Commands and terminal events are always delivered.
                if (current.State != PlaybackState.Playing || !Events.TryPeek(out _)) Publish(current);
            }
            catch (Exception error) { Fail(current.Path, error); }
        }
    }

    private void PublishTerminal(TrackOutput output)
    {
        var state = output.State;
        if (state is PlaybackState.Ended or PlaybackState.Failed && state != lastState) Publish(output);
    }

    private void Publish(TrackOutput output)
    {
        var snapshot = output.Snapshot() with { RequestId = requestId };
        if (snapshot.State == PlaybackState.Failed) failure = snapshot;
        if (snapshot.State is PlaybackState.Failed or PlaybackState.Ended)
        {
            if (lastState == snapshot.State && lastPublishedId == requestId) return;
            output.Pause();
        }
        if (snapshot.State != lastState)
            timer.Change(snapshot.State == PlaybackState.Playing ? 33 : Timeout.Infinite, 33);
        lastState = snapshot.State;
        lastPublishedId = requestId;
        events.Writer.TryWrite(snapshot);
    }

    private void Fail(string path, Exception error)
    {
        string message = error.Message;
        try { CloseTrack(); }
        catch (Exception cleanup) { message += $" ({cleanup.Message})"; }
        lastState = PlaybackState.Failed;
        failure = new PlaybackEvent(path, lastState, TimeSpan.Zero, TimeSpan.Zero, [], message, requestId);
        events.Writer.TryWrite(failure);
    }

    private void CloseTrack()
    {
        timer.Change(Timeout.Infinite, Timeout.Infinite);
        var previous = track;
        stream?.Bind(null);
        Volatile.Write(ref track, null);
        previous?.Dispose();
    }

    /// <summary>Stops callbacks, releases the file and device and completes the event stream.</summary>
    public void Dispose()
    {
        StopPlayback();
        lock (commands)
        {
            if (disposed) return;
            disposed = true;
            try { CloseTrack(); }
            finally
            {
                timer.Dispose();
                try { stream?.Dispose(); }
                finally
                {
                    try { engine?.Dispose(); }
                    finally { events.Writer.TryComplete(); }
                }
            }
        }
    }
}
