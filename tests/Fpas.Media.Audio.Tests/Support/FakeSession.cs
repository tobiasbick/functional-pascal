// [Documentation](../../../docs/pascal/libraries/media/audio.md)
using System.Threading.Channels;

namespace Fpas.Media.Audio.Tests.Support;

// A deterministic transport for application tests. The test supplies the decoder/device
// events in order, independently of the commands issued by the application.
internal sealed class FakeSession : ISession
{
    private readonly Channel<PlaybackEvent> events = Channel.CreateUnbounded<PlaybackEvent>();
    internal List<(string Operation, object? Argument)> Commands { get; } = [];
    internal void Report(PlaybackEvent update) => events.Writer.TryWrite(update);

    /// <inheritdoc />
    public ChannelReader<PlaybackEvent> Events => events.Reader;
    /// <inheritdoc />
    public void Open(string path, long requestId = 0) => Commands.Add((nameof(Open), path));
    /// <inheritdoc />
    public void OpenStream(Uri address, long requestId = 0) => Commands.Add((nameof(OpenStream), address));
    /// <inheritdoc />
    public void Play(long requestId = 0) => Commands.Add((nameof(Play), null));
    /// <inheritdoc />
    public void Pause(long requestId = 0) => Commands.Add((nameof(Pause), null));
    /// <inheritdoc />
    public void Seek(TimeSpan position, long requestId = 0) => Commands.Add((nameof(Seek), position));
    /// <inheritdoc />
    public void Unload() => Commands.Add((nameof(Unload), null));
    /// <inheritdoc />
    public PlaybackEvent? StopPlayback() { Commands.Add((nameof(StopPlayback), null)); return null; }
    /// <inheritdoc />
    public void Apply(Settings settings) => Commands.Add((nameof(Apply), settings));
    /// <inheritdoc />
    public void Dispose() => events.Writer.TryComplete();
}
