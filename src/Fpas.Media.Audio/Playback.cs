// [Documentation](../../docs/pascal/libraries/media/audio.md#playback)
using System.Collections.Immutable;
using System.Threading.Channels;

namespace Fpas.Media.Audio;

/// <summary>Transport state of one local track.</summary>
public enum PlaybackState
{
    /// <summary>The track is ready or paused.</summary>
    Paused,
    /// <summary>The output device is rendering the track.</summary>
    Playing,
    /// <summary>The decoder reached the end during playback.</summary>
    Ended,
    /// <summary>The requested file or output device could not be used.</summary>
    Failed
}

/// <summary>A snapshot of rendered audio, or a transport transition.</summary>
/// <param name="RequestId">Caller-supplied transport request that produced this observation.</param>
/// <param name="Path">Path passed to Open.</param>
/// <param name="State">Current transport state.</param>
/// <param name="Position">Elapsed time at the decoder output.</param>
/// <param name="Duration">Presentation length of the file.</param>
/// <param name="Peaks">Post-EQ, post-volume linear peaks per channel, between zero and one.</param>
/// <param name="Error">Failure description when State is Failed, otherwise null.</param>
/// <param name="Buffering">A live stream needs more audio before rendering can continue.</param>
/// <param name="NowPlaying">Optional live program or track text supplied by the stream.</param>
public sealed record PlaybackEvent(string Path, PlaybackState State, TimeSpan Position, TimeSpan Duration,
    ImmutableArray<float> Peaks, string? Error, long RequestId = 0, bool Buffering = false, string? NowPlaying = null);

/// <summary>One transport and its event stream, consumed by the owning application.</summary>
public interface ISession : IDisposable
{
    /// <summary>Ordered state changes, natural ends, failures and progress while playing.</summary>
    ChannelReader<PlaybackEvent> Events { get; }
    /// <summary>Replaces the current track and prepares it in the paused state.</summary>
    /// <param name="path">Local file to decode.</param>
    /// <param name="requestId">Caller correlation ID for this action and subsequent progress.</param>
    void Open(string path, long requestId = 0);
    /// <summary>Replace the active source with a live HTTP(S) radio stream.</summary>
    /// <param name="address">Absolute endpoint of the stream or playlist.</param>
    /// <param name="requestId">Caller correlation ID for the transport action.</param>
    void OpenStream(Uri address, long requestId = 0);
    /// <summary>Starts or resumes the open track. Replays an ended track from its beginning.</summary>
    /// <param name="requestId">Caller correlation ID for this action and subsequent progress.</param>
    void Play(long requestId = 0);
    /// <summary>Suspends rendering at the current position.</summary>
    /// <param name="requestId">Caller correlation ID for this action and subsequent progress.</param>
    void Pause(long requestId = 0);
    /// <summary>Moves to a time in the track, preserving playback or pause.</summary>
    /// <param name="position">Desired presentation time, clamped to the track boundaries.</param>
    /// <param name="requestId">Caller correlation ID for this action and subsequent progress.</param>
    void Seek(TimeSpan position, long requestId = 0);
    /// <summary>Moves to a fraction of the decoded track, preserving playback or pause.</summary>
    /// <param name="fraction">Finite progress fraction, clamped to zero through one.</param>
    /// <param name="requestId">Caller correlation ID for this action and subsequent progress.</param>
    void SeekFraction(double fraction, long requestId = 0);
    /// <summary>Stops playback and releases the current track. The session can open another file.</summary>
    void Unload();
    /// <summary>Silences the session permanently and cancels ongoing decoder work before GUI cleanup.</summary>
    /// <returns>The last rendered track position, or null if no track is open.</returns>
    PlaybackEvent? StopPlayback();
    /// <summary>Applies an immutable mixer snapshot to this and subsequent tracks.</summary>
    /// <param name="settings">Volume and ten equalizer gains in dB.</param>
    void Apply(Settings settings);
}
