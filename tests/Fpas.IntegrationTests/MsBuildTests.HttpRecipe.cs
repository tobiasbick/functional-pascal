using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Documented_http_recipe_reads_bounds_cancels_and_disposes_owned_messages(string configuration)
    {
        using var project = sdk.CreateProject();
        var document = Path.Combine(AppContext.BaseDirectory, "Fixtures", "specification", "libraries", "http.md");
        var unit = MarkdownExamples.Read(document).Single(example => example.Library);
        File.WriteAllText(project.File("Game/Bridge.fpas"), unit.Source);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using System.Net;
            using System.Net.Sockets;
            using System.Text;
            using Fetch = Recipes.HttpText;

            static void Check(bool condition, string message)
            {
                if (!condition) throw new Exception(message);
            }
            static async Task<T> Fails<T>(Func<Task> action) where T : Exception
            {
                try { await action(); }
                catch (T failure) { return failure; }
                throw new Exception("Expected " + typeof(T).Name);
            }

            // A real socket exchange verifies the documented UTF-8 GET path without ambient proxies.
            using (var listener = new TcpListener(IPAddress.Loopback, 0))
            using (var stop = new CancellationTokenSource(10000))
            using (var client = new HttpClient(new HttpClientHandler { UseProxy = false }))
            {
                listener.Start();
                var uri = new Uri("http://127.0.0.1:" + ((IPEndPoint)listener.LocalEndpoint).Port + "/text");
                var expected = "Hello, café 😀";
                async Task Serve()
                {
                    using var connection = await listener.AcceptTcpClientAsync(stop.Token);
                    using var stream = connection.GetStream();
                    using var reader = new StreamReader(stream, Encoding.ASCII, leaveOpen: true);
                    Check(await reader.ReadLineAsync(stop.Token) == "GET /text HTTP/1.1", "Incorrect request target");
                    var headers = new List<string>();
                    while (await reader.ReadLineAsync(stop.Token) is { Length: > 0 } line) headers.Add(line);
                    Check(headers.Any(line => line.Equals("Accept: text/plain", StringComparison.OrdinalIgnoreCase)), "Missing Accept header");
                    var body = Encoding.UTF8.GetBytes(expected);
                    var header = Encoding.ASCII.GetBytes("HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: " +
                        body.Length + "\r\nConnection: close\r\n\r\n");
                    await stream.WriteAsync(header, stop.Token);
                    await stream.WriteAsync(body.AsMemory(0, body.Length - 1), stop.Token);
                    await stream.WriteAsync(body.AsMemory(body.Length - 1), stop.Token);
                }
                var serving = Serve();
                try
                {
                    Check(await Fetch.GetUtf8(client, uri, expected.Length, stop.Token) == expected, "UTF-8 body changed");
                    await serving;
                }
                finally
                {
                    stop.Cancel();
                    try { await serving; }
                    catch (OperationCanceledException) when (stop.IsCancellationRequested) { }
                }
            }

            using var handler = new ProbeHandler();
            using var reusable = new HttpClient(handler) { Timeout = Timeout.InfiniteTimeSpan };
            var address = new Uri("https://example.com/text");
            async Task Disposed()
            {
                Check(handler.Request is not null, "Request did not reach the handler");
                using var content = new ByteArrayContent([]);
                await Fails<ObjectDisposedException>(() =>
                {
                    handler.Request!.Content = content;
                    return Task.CompletedTask;
                });
                if (handler.Response is not null)
                {
                    await Fails<ObjectDisposedException>(() => handler.Response.Content.ReadAsStreamAsync());
                    Check(handler.Body!.Disposed, "Response body remained open");
                }
                Check(!handler.Disposed, "Helper disposed the caller's client/handler");
            }

            foreach (var text in new[] { "é😀", "", new string('x', 5000) })
            {
                handler.Reset(Encoding.UTF8.GetBytes(text));
                Check(await Fetch.GetUtf8(reusable, address, text.Length, CancellationToken.None) == text, "Exact limit failed");
                await Disposed();
            }
            handler.Reset(Encoding.UTF8.GetBytes("123456789"));
            await Fails<InvalidDataException>(() => Fetch.GetUtf8(reusable, address, 8, CancellationToken.None));
            await Disposed();
            handler.Reset([0xc3, 0x28]);
            await Fails<DecoderFallbackException>(() => Fetch.GetUtf8(reusable, address, 100, CancellationToken.None));
            await Disposed();
            handler.Reset([]);
            handler.Status = HttpStatusCode.NotFound;
            var status = await Fails<HttpRequestException>(() => Fetch.GetUtf8(reusable, address, 100, CancellationToken.None));
            Check(status.StatusCode == HttpStatusCode.NotFound, "HTTP status was lost");
            await Disposed();

            foreach (var phase in new[] { "headers", "body" })
            {
                handler.Reset([]);
                handler.BlockHeaders = phase == "headers";
                handler.BlockBody = phase == "body";
                using var cancellation = new CancellationTokenSource();
                var pending = Fetch.GetUtf8(reusable, address, 100, cancellation.Token);
                try
                {
                    await (phase == "headers" ? handler.HeaderStarted.Task : handler.ReadStarted.Task).WaitAsync(TimeSpan.FromSeconds(5));
                    cancellation.Cancel();
                    await Fails<OperationCanceledException>(() => pending.WaitAsync(TimeSpan.FromSeconds(5)));
                }
                finally
                {
                    cancellation.Cancel();
                    handler.Release.TrySetResult();
                    try { await pending.WaitAsync(TimeSpan.FromSeconds(5)); }
                    catch (OperationCanceledException) { }
                }
                await Disposed();
            }
            Console.WriteLine("HTTP recipe verified");

            sealed class ProbeHandler : HttpMessageHandler
            {
                public HttpRequestMessage? Request;
                public HttpResponseMessage? Response;
                public ProbeStream? Body;
                public HttpStatusCode Status;
                public bool BlockHeaders, BlockBody, Disposed;
                public byte[] Payload = [];
                public TaskCompletionSource HeaderStarted = new(TaskCreationOptions.RunContinuationsAsynchronously);
                public TaskCompletionSource ReadStarted = new(TaskCreationOptions.RunContinuationsAsynchronously);
                public TaskCompletionSource Release = new(TaskCreationOptions.RunContinuationsAsynchronously);
                public void Reset(byte[] payload)
                {
                    Payload = payload; Status = HttpStatusCode.OK;
                    Request = null; Response = null; Body = null; BlockHeaders = BlockBody = false;
                    HeaderStarted = new(TaskCreationOptions.RunContinuationsAsynchronously);
                    ReadStarted = new(TaskCreationOptions.RunContinuationsAsynchronously);
                    Release = new(TaskCreationOptions.RunContinuationsAsynchronously);
                }
                protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
                {
                    Request = request;
                    HeaderStarted.TrySetResult();
                    if (BlockHeaders) await Release.Task.WaitAsync(cancellationToken);
                    Body = new ProbeStream(Payload, ReadStarted, Release, BlockBody);
                    Response = new HttpResponseMessage(Status) { Content = new StreamContent(Body), RequestMessage = request };
                    return Response;
                }
                protected override void Dispose(bool disposing)
                {
                    Disposed = true;
                    base.Dispose(disposing);
                }
            }

            sealed class ProbeStream(byte[] payload, TaskCompletionSource started, TaskCompletionSource release, bool block)
                : MemoryStream(payload)
            {
                public bool Disposed;
                public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancellationToken = default)
                {
                    started.TrySetResult();
                    if (block) await release.Task.WaitAsync(cancellationToken);
                    return await base.ReadAsync(buffer, cancellationToken);
                }
                public override Task<int> ReadAsync(byte[] buffer, int offset, int count, CancellationToken cancellationToken)
                    => ReadAsync(buffer.AsMemory(offset, count), cancellationToken).AsTask();
                protected override void Dispose(bool disposing)
                {
                    Disposed = true;
                    base.Dispose(disposing);
                }
            }
            """);
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration, "-warnaserror");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await ProcessRunner.RunAsync([project.File($"Host/bin/{configuration}/net10.0/Host.dll")],
            workingDirectory: project.Path, timeLimit: TimeSpan.FromSeconds(40));
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("HTTP recipe verified" + Environment.NewLine, run.Output);
        Assert.Empty(run.Error);
    }
}
