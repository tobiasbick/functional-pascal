// [Documentation](../../doc/media/audio.md#verification)
using System.Net;
using System.Net.Sockets;
using FpasAmp.Media.Audio.Tests.Support;

namespace FpasAmp.Media.Audio.Tests;

/// <summary>Exercises an indefinite HTTP radio stream through the real decoder and mixer.</summary>
public sealed class LiveStreamsTests
{
    private sealed class RequiresFfmpegAttribute : FactAttribute
    {
        public RequiresFfmpegAttribute()
        {
            string name = OperatingSystem.IsWindows() ? "ffmpeg.exe" : "ffmpeg";
            if (!(Environment.GetEnvironmentVariable("PATH") ?? "").Split(Path.PathSeparator)
                .Any(directory => directory.Length > 0 && File.Exists(Path.Combine(directory.Trim('"'), name))))
                Skip = "FFmpeg is required to exercise the radio decoder.";
        }
    }

    /// <summary>Checks buffering, ICY metadata, connection loss and a return to local playback.</summary>
    [RequiresFfmpeg]
    public async Task RendersChunkedRadioAndRecoversAfterDisconnect()
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(20));
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        var address = new Uri($"http://{listener.LocalEndpoint}/radio");
        string file = Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.mp3");
        byte[] mp3 = await File.ReadAllBytesAsync(file, timeout.Token);
        var disconnect = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        Task serving = Serve(listener, mp3, disconnect.Task, timeout.Token);
        using var session = new Session(true);
        try
        {
            session.OpenStream(address, 1);
            session.Play(1);
            await Find(session, e => e.RequestId == 1 && e.Buffering, timeout.Token);
            var heard = await Find(session, e => e.RequestId == 1 && e.State == PlaybackState.Playing
                && !e.Buffering && e.NowPlaying == "Local Radio Test" && e.Peaks.Any(peak => peak > 0), timeout.Token);
            Assert.Equal(TimeSpan.Zero, heard.Duration);
            disconnect.SetResult();
            var failure = await Find(session, e => e.RequestId == 1 && e.State == PlaybackState.Failed, timeout.Token);
            Assert.Contains("disconnected", failure.Error);
            session.Unload();
            session.Open(file, 2);
            session.Play(2);
            await Find(session, e => e.RequestId == 2 && e.State == PlaybackState.Playing
                && e.Peaks.Any(peak => peak > 0), timeout.Token);
        }
        finally
        {
            await timeout.CancelAsync();
            try { await serving; }
            catch (OperationCanceledException) { }
        }
    }

    /// <summary>Stopping the session closes the radio connection before the session is disposed.</summary>
    [RequiresFfmpeg]
    public async Task StopPlaybackClosesRadioConnection()
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(20));
        using var listener = new TcpListener(IPAddress.Loopback, 0);
        listener.Start();
        byte[] mp3 = await File.ReadAllBytesAsync(Path.Combine(AppContext.BaseDirectory, "Fixtures", "step.mp3"), timeout.Token);
        var disconnect = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        Task serving = Serve(listener, mp3, disconnect.Task, timeout.Token);
        using var session = new Session(true);
        try
        {
            session.OpenStream(new Uri($"http://{listener.LocalEndpoint}/radio"));
            session.Play();
            await Find(session, e => e.State == PlaybackState.Playing && !e.Buffering
                && e.Peaks.Any(peak => peak > 0), timeout.Token);
            session.StopPlayback();
            try { await serving.WaitAsync(timeout.Token); }
            catch (IOException) { }
            catch (SocketException) { }
        }
        finally
        {
            await timeout.CancelAsync();
            try { await serving; }
            catch (OperationCanceledException) { }
            catch (IOException) { }
            catch (SocketException) { }
        }
    }

    private static async Task<PlaybackEvent> Find(Session session, Func<PlaybackEvent, bool> check, CancellationToken cancellation)
    {
        await foreach (PlaybackEvent update in session.Events.ReadAllAsync(cancellation))
            if (check(update)) return update;
        throw new InvalidOperationException("Playback events ended early.");
    }

    private static async Task Serve(TcpListener listener, byte[] mp3, Task disconnect, CancellationToken cancellation)
    {
        using var client = await listener.AcceptTcpClientAsync(cancellation);
        using NetworkStream stream = client.GetStream();
        using var request = new StreamReader(stream, leaveOpen: true);
        while (await request.ReadLineAsync(cancellation) is { Length: > 0 }) { }
        await stream.WriteAsync("HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nTransfer-Encoding: chunked\r\nicy-metaint: 512\r\nConnection: close\r\n\r\n"u8.ToArray(), cancellation);
        // Each HTTP chunk carries 512 MP3 bytes, then one padded ICY title block.
        byte[] chunk = new byte[517 + 33 + 2];
        "221\r\n"u8.CopyTo(chunk);
        chunk[517] = 2;
        "StreamTitle='Local Radio Test';"u8.CopyTo(chunk.AsSpan(518));
        "\r\n"u8.CopyTo(chunk.AsSpan(chunk.Length - 2));
        var offset = 0;
        while (!disconnect.IsCompleted)
        {
            for (int i = 5; i < 517; i++)
            {
                chunk[i] = mp3[offset++];
                if (offset == mp3.Length) offset = 0;
            }
            await stream.WriteAsync(chunk, cancellation);
            await Task.Delay(4, cancellation);
        }
        await stream.WriteAsync("0\r\n\r\n"u8.ToArray(), cancellation);
    }
}
