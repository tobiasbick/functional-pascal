using Fpas.Debugging.Protocol;
using Fpas.Debugging.Sessions;
using Fpas.Tests.Support;
using System.Text;

namespace Fpas.IntegrationTests.Debugging;

public sealed class McpInfrastructureTests
{
    [Fact]
    public void Output_pages_preserve_UTF8_and_explicitly_report_eviction_and_invalid_cursors()
    {
        var output = new OutputBuffer();
        var text = string.Concat(Enumerable.Repeat("A🙂ä", 200_000));
        output.Add("stdout", text);
        var page = output.Read(0, 32768);
        Assert.True(page.Truncated);
        var retained = new StringBuilder(); long cursor = 0; var bytes = 0;
        do
        {
            page = output.Read(cursor, 32768);
            var value = string.Concat(page.Items.Select(item => item.Text));
            Assert.InRange(Encoding.UTF8.GetByteCount(value), 1, 32768);
            Assert.DoesNotContain('�', value);
            retained.Append(value); bytes += Encoding.UTF8.GetByteCount(value);
            Assert.True(page.NextSequence > cursor); cursor = page.NextSequence;
        } while (page.More);
        Assert.InRange(bytes, 1, 1_048_576);
        Assert.EndsWith(retained.ToString(), text);
        Assert.Empty(output.Read(cursor, 32768).Items);
        Assert.Equal("invalid_argument", Assert.Throws<DebugFailure>(() => output.Read(cursor + 1, 32768)).Code);
        Assert.Equal("limit_too_small", Assert.Throws<DebugFailure>(() => output.Read(0, 1)).Code);
    }

    [LinuxFact]
    public async Task Malformed_adapter_frame_completes_pending_requests_and_cleanup()
    {
        using var fixture = new TemporaryDirectory();
        var path = fixture.File("backend");
        File.WriteAllText(path, "#!/bin/sh\nprintf 'Content-Length: 2097152\\r\\n\\r\\n'\nsleep 60\n");
        if (OperatingSystem.IsLinux()) File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        var failed = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
        await using var adapter = new DapConnection(path, fixture.Path, _ => { }, _ => failed.TrySetResult());
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(5));
        var failure = await Assert.ThrowsAsync<DebugFailure>(() => adapter.Initialize(timeout.Token));
        Assert.Equal("backend_exited", failure.Code);
        await failed.Task.WaitAsync(timeout.Token);
    }

    [LinuxFact]
    public async Task Unresponsive_adapter_deadline_completes_outstanding_requests()
    {
        using var fixture = new TemporaryDirectory();
        var path = fixture.File("backend");
        File.WriteAllText(path, "#!/bin/sh\nsleep 60\n");
        if (OperatingSystem.IsLinux()) File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        await using var adapter = new DapConnection(path, fixture.Path, _ => { }, _ => { });
        var failure = await Assert.ThrowsAsync<DebugFailure>(() => adapter.Request("launch", new { payload = new string('x', 200_000) }, CancellationToken.None));
        Assert.Equal("backend_timeout", failure.Code);
    }

    [LinuxFact]
    public async Task Concurrent_disposal_completes_outstanding_requests_before_releasing_resources()
    {
        using var fixture = new TemporaryDirectory();
        var path = fixture.File("backend");
        File.WriteAllText(path, "#!/bin/sh\nsleep 60\n");
        if (OperatingSystem.IsLinux()) File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        await using var adapter = new DapConnection(path, fixture.Path, _ => { }, _ => { });
        var requests = Enumerable.Range(0, 4).Select(_ => adapter.Request("threads", new { }, CancellationToken.None)).ToArray();
        await Task.WhenAll(adapter.DisposeAsync().AsTask(), adapter.DisposeAsync().AsTask());
        foreach (var request in requests)
            Assert.Equal("backend_exited", (await Assert.ThrowsAsync<DebugFailure>(() => request)).Code);
    }
}

internal sealed class LinuxFactAttribute : FactAttribute
{
    public LinuxFactAttribute()
    {
        if (!OperatingSystem.IsLinux()) Skip = "Linux process-group ownership regression.";
    }
}
