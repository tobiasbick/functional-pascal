// [Documentation](../../../doc/media/audio.md#local-formats-and-seek-behavior)
using SoundFlow.Backends.MiniAudio;
using SoundFlow.Codecs.FFMpeg;
using SoundFlow.Enums;
using SoundFlow.Interfaces;
using SoundFlow.Metadata.Models;
using SoundFlow.Structs;

namespace FpasAmp.Media.Audio.Decoding;

// One owner reads and positions this file. The mixer must use Format as its client format.
internal sealed class TrackSource : ISoundDataProvider, IPlaybackSource
{
    private static readonly MiniAudioCodecFactory BuiltIn = new();
    private static readonly FFmpegCodecFactory FFmpeg = new();
    private static readonly AudioFormat Stereo = new() { Format = SampleFormat.F32, SampleRate = 44100, Channels = 2 };
    private readonly FileStream stream;
    private readonly string extension;
    private readonly bool useBuiltIn;
    private readonly float[] scratch;
    private readonly CancellationToken cancellation;
    private ISoundDecoder decoder;

    internal TrackSource(string path, CancellationToken cancellation = default)
    {
        cancellation.ThrowIfCancellationRequested();
        this.cancellation = cancellation;
        extension = Path.GetExtension(path).TrimStart('.').ToLowerInvariant();
        useBuiltIn = BuiltIn.SupportedFormatIds.Contains(extension);
        if (!useBuiltIn && extension is not ("m4a" or "ogg" or "opus"))
            throw new NotSupportedException($"Unsupported audio format '{extension}'.");

        stream = File.OpenRead(path);
        try
        {
            decoder = OpenDecoder();
            Format = new AudioFormat { Format = SampleFormat.F32, SampleRate = decoder.SampleRate, Channels = decoder.Channels };
            scratch = new float[4096 * Format.Channels];
            Length = decoder.Length;
            // Ogg timestamps can include pre-skip. Count audible samples when the header
            // cannot supply the exact length needed for transport and waveform buckets.
            if (extension is "ogg" or "opus" || Length <= 0)
            {
                int length = 0;
                int read;
                while ((read = decoder.Decode(scratch)) > 0)
                {
                    cancellation.ThrowIfCancellationRequested();
                    length = checked(length + read);
                }
                Length = length;
                Restart();
            }
        }
        catch
        {
            decoder?.Dispose();
            stream.Dispose();
            throw;
        }
    }

    internal AudioFormat Format { get; }
    AudioFormat IPlaybackSource.Format => Format;
    long IPlaybackSource.Position => Position;
    bool IPlaybackSource.Live => false;
    string? IPlaybackSource.NowPlaying => null;
    /// <inheritdoc />
    public int Length { get; }
    /// <inheritdoc />
    public int Position { get; private set; }
    /// <inheritdoc />
    public int SampleRate => Format.SampleRate;
    /// <inheritdoc />
    public SampleFormat SampleFormat => SampleFormat.F32;
    /// <inheritdoc />
    public bool CanSeek => !IsDisposed;
    /// <inheritdoc />
    public bool IsDisposed { get; private set; }
    /// <inheritdoc />
    public SoundFormatInfo? FormatInfo => null;
    /// <inheritdoc />
    public event EventHandler<EventArgs>? EndOfStreamReached;
    /// <inheritdoc />
    public event EventHandler<PositionChangedEventArgs>? PositionChanged;

    /// <inheritdoc />
    public int ReadBytes(Span<float> buffer)
    {
        ObjectDisposedException.ThrowIf(IsDisposed, this);
        if (buffer.Length < Format.Channels) return 0;
        // Some AAC encoders pad the last packet. The container length excludes this tail.
        int count = Math.Min(buffer.Length, Length - Position);
        int read = count > 0 ? decoder.Decode(buffer[..count]) : 0;
        Position += read;
        if (read == 0) EndOfStreamReached?.Invoke(this, EventArgs.Empty);
        return read;
    }

    /// <inheritdoc />
    public void Seek(int sampleOffset)
    {
        ObjectDisposedException.ThrowIf(IsDisposed, this);
        int target = Math.Clamp(sampleOffset, 0, Length);
        target -= target % Format.Channels;
        if (target == Position) return;
        if (extension is ("wav" or "flac") && decoder.Seek(target))
            Position = target;
        else
        {
            // Packet seeks in FFmpeg and MP3 can miss the target. Continuous decoding preserves the timeline,
            // including codec priming when seeking backwards across the first packet.
            if (target < Position) Restart();
            while (Position < target)
            {
                cancellation.ThrowIfCancellationRequested();
                int read = decoder.Decode(scratch.AsSpan(0, Math.Min(scratch.Length, target - Position)));
                if (read == 0) break;
                Position += read;
            }
        }
        PositionChanged?.Invoke(this, new PositionChangedEventArgs(Position));
    }

    private ISoundDecoder OpenDecoder() => (useBuiltIn
        ? BuiltIn.CreateDecoder(stream, extension, Stereo)
        : FFmpeg.CreateDecoder(stream, extension, Stereo)) ?? throw new InvalidDataException("Cannot decode audio file.");

    private void Restart()
    {
        decoder.Dispose();
        stream.Position = 0;
        decoder = OpenDecoder();
        Position = 0;
    }

    /// <inheritdoc />
    public void Dispose()
    {
        if (IsDisposed) return;
        IsDisposed = true;
        decoder.Dispose();
        stream.Dispose();
    }
}
