// [Documentation](../../../docs/pascal/libraries/media/audio.md#live-streams)
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Channels;
using SoundFlow.Enums;
using SoundFlow.Structs;

namespace Fpas.Media.Audio.Decoding;

// FFmpeg handles HTTP, playlist demuxing and codecs. The device only reads a bounded PCM queue.
internal sealed class LiveSource : IPlaybackSource
{
    private const int Block = 2048;
    private readonly Process process;
    private readonly CancellationTokenSource stop;
    private readonly Channel<float[]> samples = Channel.CreateBounded<float[]>(12);
    private readonly Task reader;
    private float[]? current;
    private int offset;
    private long position;
    private string? nowPlaying;

    internal LiveSource(Uri address, CancellationToken cancellation)
    {
        var info = new ProcessStartInfo
        {
            FileName = "ffmpeg",
            UseShellExecute = false,
            CreateNoWindow = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardErrorEncoding = System.Text.Encoding.UTF8
        };
        foreach (string argument in new[]
        {
            "-nostdin", "-hide_banner", "-loglevel", "verbose", "-rw_timeout", "8000000",
            "-analyzeduration", "1000000",
            "-protocol_whitelist", "http,https,tcp,tls,httpproxy,crypto", "-i", address.AbsoluteUri,
            "-map", "0:a:0", "-vn", "-sn", "-dn", "-ac", "2", "-ar", "44100", "-f", "f32le", "pipe:1"
        }) info.ArgumentList.Add(argument);
        process = new Process { StartInfo = info };
        stop = CancellationTokenSource.CreateLinkedTokenSource(cancellation);
        try
        {
            stop.Token.ThrowIfCancellationRequested();
            process.ErrorDataReceived += ReceiveMetadata;
            process.Start();
            process.BeginErrorReadLine();
            reader = PumpAsync(stop.Token);
        }
        catch (System.ComponentModel.Win32Exception error)
        {
            process.Dispose();
            stop.Dispose();
            throw new IOException("Radio requires FFmpeg on PATH.", error);
        }
        catch
        {
            StopProcess();
            process.Dispose();
            stop.Dispose();
            throw;
        }
    }

    public AudioFormat Format { get; } = new() { SampleRate = 44100, Channels = 2, Format = SampleFormat.F32 };
    public int Length => 0;
    public long Position => position;
    public bool Live => true;
    public string? NowPlaying => Volatile.Read(ref nowPlaying);

    public int ReadBytes(Span<float> output)
    {
        int written = 0;
        while (!output.IsEmpty)
        {
            if (current is null)
            {
                if (!samples.Reader.TryRead(out current)) break;
                offset = 0;
            }
            int count = Math.Min(output.Length, current.Length - offset);
            current.AsSpan(offset, count).CopyTo(output);
            output = output[count..];
            offset += count;
            written += count;
            if (offset == current.Length) current = null;
        }

        position += written;
        if (written == 0 && samples.Reader.Completion.IsCompleted && !stop.IsCancellationRequested)
            throw new IOException("The station disconnected. Press Play to reconnect.");
        return written;
    }

    public void Seek(int sampleOffset) => throw new NotSupportedException();

    private async Task PumpAsync(CancellationToken cancellation)
    {
        var buffer = new byte[Block * sizeof(float)];
        try
        {
            Stream output = process.StandardOutput.BaseStream;
            while (true)
            {
                int count = await output.ReadAtLeastAsync(buffer, buffer.Length, false, cancellation).ConfigureAwait(false);
                count -= count % (2 * sizeof(float));
                if (count == 0) break;
                var pcm = MemoryMarshal.Cast<byte, float>(buffer.AsSpan(0, count)).ToArray();
                await samples.Writer.WriteAsync(pcm, cancellation).ConfigureAwait(false);
            }
        }
        catch (Exception error) when (error is OperationCanceledException or IOException or ObjectDisposedException)
        {
            // The receiver reports disconnect after the last playable buffer.
        }
        finally { samples.Writer.TryComplete(); }
    }

    private void ReceiveMetadata(object sender, DataReceivedEventArgs args)
    {
        const string Marker = "Metadata update for StreamTitle: ";
        string? line = args.Data;
        if (line is null) return;
        int start = line.IndexOf(Marker, StringComparison.OrdinalIgnoreCase);
        if (start < 0) return;
        start += Marker.Length;
        if (line.Length - start <= 512)
            Volatile.Write(ref nowPlaying, line[start..].Trim());
    }

    private void StopProcess()
    {
        try { if (!process.HasExited) process.Kill(); }
        catch (InvalidOperationException) { }
    }

    public void Dispose()
    {
        stop.Cancel();
        StopProcess();
        reader.GetAwaiter().GetResult();
        process.Dispose();
        stop.Dispose();
    }
}
