using Fpas.Debugging.Processes;
using Fpas.Debugging.Sessions;
using Fpas.Debugging.Tools;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;

namespace Fpas.Debugging;

public static class DebugServer
{
    public static async Task RunAsync(string backendPath, CancellationToken cancellationToken = default)
    {
        OwnedBackend.Validate(backendPath);
        var builder = Host.CreateApplicationBuilder(new HostApplicationBuilderSettings { DisableDefaults = true });
        builder.Logging.AddConsole(options => options.LogToStandardErrorThreshold = LogLevel.Trace);
        builder.Logging.SetMinimumLevel(LogLevel.Warning);
        var session = new DebugSession(backendPath);
        builder.Services.AddSingleton(session);
        builder.Services.AddMcpServer().WithStdioServerTransport().WithTools<DebugTools>();
        using var host = builder.Build();
        try { await host.RunAsync(cancellationToken); }
        finally { await session.DisposeAsync(); }
    }
}
