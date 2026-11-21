// [Documentation](../../../docs/pascal/libraries/media/audio.md#playback)
using SoundFlow.Structs;

namespace Fpas.Media.Audio.Decoding;

internal interface IPlaybackSource : IDisposable
{
    AudioFormat Format { get; }
    int Length { get; }
    long Position { get; }
    bool Live { get; }
    string? NowPlaying { get; }
    int ReadBytes(Span<float> output);
    void Seek(int sampleOffset);
}
