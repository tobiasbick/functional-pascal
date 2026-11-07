using Fpas.Tests.Support;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Globalization;
using System.Text;
using System.Text.Json;
using System.Threading.Channels;

namespace Fpas.IntegrationTests.LanguageServer;

// Independent JSON-RPC/stdio client; no product framing, DTOs or server handles.
internal sealed class LspProbe : IAsyncDisposable
{
    private readonly Process process;
    private readonly CancellationTokenSource lifetime = new(TimeSpan.FromMinutes(2));
    private readonly SemaphoreSlim writes = new(1);
    private readonly ConcurrentDictionary<string, TaskCompletionSource<JsonElement>> pending = [];
    private readonly Channel<JsonElement> notifications = Channel.CreateBounded<JsonElement>(1024);
    private readonly ConcurrentQueue<string> received = new();
    private readonly Task reading;
    private readonly Task<string> errors;
    private int sequence;
    internal int Id => process.Id;
    internal Task<string> Error => errors;

    internal LspProbe(string? cli = null, bool executable = false, string? directory = null, IReadOnlyDictionary<string, string?>? environment = null)
    {
        var start = new ProcessStartInfo(executable ? cli! : TestEnvironment.DotnetHost) { RedirectStandardInput = true, RedirectStandardOutput = true,
            RedirectStandardError = true, UseShellExecute = false, CreateNoWindow = true };
        if (!executable) start.ArgumentList.Add(cli ?? TestEnvironment.Metadata("FpasCliPath"));
        start.ArgumentList.Add("lsp");
        if (directory is not null) start.WorkingDirectory = directory;
        if (environment is not null) foreach (var (key, value) in environment) start.Environment[key] = value;
        start.Environment["MSBUILDDISABLENODEREUSE"] = "1"; start.Environment["DOTNET_CLI_USE_MSBUILD_SERVER"] = "0";
        process = Process.Start(start)!; errors = process.StandardError.ReadToEndAsync(lifetime.Token); reading = Read();
    }

    internal Task<JsonElement> Request(string method, object? parameters = null, string? stringId = null)
    {
        object id = stringId is null ? Interlocked.Increment(ref sequence) : stringId;
        var source = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        Assert.True(pending.TryAdd(Key(id), source));
        return SendAndWait();
        async Task<JsonElement> SendAndWait()
        { await Send(new { jsonrpc = "2.0", id, method, @params = parameters }); return await source.Task.WaitAsync(lifetime.Token); }
    }
    internal Task Notify(string method, object? parameters = null) => Send(new { jsonrpc = "2.0", method, @params = parameters });
    internal Task Reply(JsonElement request, object? result = null) => Send(new { jsonrpc = "2.0", id = request.GetProperty("id"), result });
    internal async Task Initialize(string context, bool watch = false, bool rename = true)
    { var response = await InitializeResponse(context, watch, rename); Assert.True(response.TryGetProperty("result", out _), response.GetRawText()); }
    internal Task<JsonElement> InitializeResponse(string context, bool watch = false, bool rename = true) => Request("initialize", new
    {
        capabilities = new { general = new { positionEncodings = new[] { "utf-16", "utf-8" } },
            workspace = new { didChangeWatchedFiles = new { dynamicRegistration = watch }, workspaceEdit = new { documentChanges = rename } },
            textDocument = new { rename = new { prepareSupport = rename }, documentSymbol = new { hierarchicalDocumentSymbolSupport = true },
                semanticTokens = new { requests = new { full = true }, tokenTypes = new[] { "namespace", "struct", "function", "variable", "parameter", "property", "type" },
                    tokenModifiers = new[] { "declaration" } } } },
        initializationOptions = new { context, configuration = "Debug" }
    });
    internal async Task<JsonElement> Notification(string method, Func<JsonElement, bool>? predicate = null)
    {
        try
        {
            await foreach (var item in notifications.Reader.ReadAllAsync(lifetime.Token))
                if (item.GetProperty("method").GetString() == method && (predicate is null || predicate(item))) return item;
        }
        catch (OperationCanceledException)
        { throw new TimeoutException("Waiting for " + method + "; received: " + string.Join("; ", received)); }
        throw new IOException("Server EOF before expected notification: " + method + " " + await errors);
    }
    internal Task<JsonElement> Diagnostics(string path, int version) => Notification("textDocument/publishDiagnostics", item =>
        IsFileUri(item.GetProperty("params").GetProperty("uri"), path) &&
        item.GetProperty("params").TryGetProperty("version", out var actual) && actual.GetInt32() == version);
    internal static bool IsFileUri(JsonElement value, string path) =>
        Uri.TryCreate(value.GetString(), UriKind.Absolute, out var uri) && uri.IsFile && !uri.IsUnc &&
        (OperatingSystem.IsWindows() ? StringComparer.OrdinalIgnoreCase : StringComparer.Ordinal).Equals(Path.GetFullPath(uri.LocalPath), Path.GetFullPath(path));
    internal static void AssertFileUri(string path, JsonElement value) =>
        Assert.True(IsFileUri(value, path), "Expected file " + path + "; received URI " + value.GetRawText());
    internal Task Open(string path, string text, int version = 1) => Notify("textDocument/didOpen", new { textDocument = new { uri = new Uri(path).AbsoluteUri, languageId = "fpas", version, text } });
    internal Task FullChange(string path, string text, int version) => Notify("textDocument/didChange", new { textDocument = new { uri = new Uri(path).AbsoluteUri, version }, contentChanges = new[] { new { text } } });
    internal Task Close(string path) => Notify("textDocument/didClose", new { textDocument = new { uri = new Uri(path).AbsoluteUri } });
    internal async Task Stop()
    {
        var response = await Request("shutdown"); Assert.Equal(JsonValueKind.Null, response.GetProperty("result").ValueKind);
        await Notify("exit"); Assert.Equal(0, await Exit()); Assert.Equal("", await errors);
    }
    internal async Task<int> Exit() { await process.WaitForExitAsync(lifetime.Token); await reading; return process.ExitCode; }
    internal void Eof() => process.StandardInput.Close();
    internal async Task Raw(byte[] bytes) { await process.StandardInput.BaseStream.WriteAsync(bytes, lifetime.Token); await process.StandardInput.BaseStream.FlushAsync(lifetime.Token); }
    private async Task Send(object value)
    {
        var bytes = JsonSerializer.SerializeToUtf8Bytes(value);
        await writes.WaitAsync(lifetime.Token);
        try { await Raw(Encoding.ASCII.GetBytes("Content-Length: " + bytes.Length.ToString(CultureInfo.InvariantCulture) + "\r\n\r\n")); await Raw(bytes); }
        finally { writes.Release(); }
    }

    private async Task Read()
    {
        Exception? failure = null;
        try
        {
            var stream = process.StandardOutput.BaseStream;
            while (true)
            {
                var header = new StringBuilder(); var single = new byte[1];
                while (!header.ToString().EndsWith("\r\n\r\n", StringComparison.Ordinal))
                {
                    var count = await stream.ReadAsync(single, lifetime.Token);
                    if (count == 0) { if (header.Length != 0) throw new IOException("Truncated server header."); return; }
                    if (header.Length >= 8192 || single[0] > 127) throw new IOException("Invalid protocol stdout.");
                    header.Append((char)single[0]);
                }
                var lengthLine = header.ToString().Split("\r\n").Single(line => line.StartsWith("Content-Length:", StringComparison.OrdinalIgnoreCase));
                var length = int.Parse(lengthLine[15..].Trim(), CultureInfo.InvariantCulture);
                if (length is < 1 or > 4_194_304) throw new IOException("Server message exceeds its budget.");
                var body = new byte[length]; await stream.ReadExactlyAsync(body, lifetime.Token);
                using var json = JsonDocument.Parse(body); var message = json.RootElement.Clone();
                Assert.Equal("2.0", message.GetProperty("jsonrpc").GetString());
                if (!message.TryGetProperty("method", out _) && message.TryGetProperty("id", out var id) && pending.TryRemove(Key(id), out var source)) source.TrySetResult(message);
                else
                {
                    received.Enqueue(message.ToString()); while (received.Count > 10) received.TryDequeue(out _);
                    await notifications.Writer.WriteAsync(message, lifetime.Token);
                }
            }
        }
        catch (Exception error) { failure = error; }
        finally
        {
            foreach (var source in pending.Values) source.TrySetException(failure ?? new IOException("Server disconnected. " + (errors.IsCompletedSuccessfully ? errors.Result : "")));
            notifications.Writer.TryComplete(failure);
        }
    }
    private static string Key(object id) => id is JsonElement element ? element.ValueKind == JsonValueKind.String ? "s:" + element.GetString() : "n:" + element.GetRawText()
        : id is string text ? "s:" + text : "n:" + Convert.ToString(id, CultureInfo.InvariantCulture);
    public async ValueTask DisposeAsync()
    {
        if (!process.HasExited) process.Kill(entireProcessTree: true);
        await process.WaitForExitAsync(); lifetime.Cancel();
        try { await reading; await errors; } catch (OperationCanceledException) { }
        process.Dispose(); writes.Dispose(); lifetime.Dispose();
    }
}
