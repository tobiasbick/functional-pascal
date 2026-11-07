using Fpas.Processes;
using Fpas.Tests.Support;
using System.Diagnostics;

namespace Fpas.IntegrationTests.Processes;

public sealed class OwnedProcessTests
{
    [Fact]
    public async Task Disposal_joins_a_child_after_its_parent_exits_without_reporting_it()
    {
        using var fixture = new TemporaryDirectory();
        var source = fixture.File("Parent.fpas"); var assembly = fixture.File("Parent.dll");
        File.WriteAllText(source, """
            program Parent;
            uses System as S, System.IO.File as File, System.Diagnostics as P,
              System.Threading.Thread as Thread;
            begin
              const Args: array of string := S.Environment.GetCommandLineArgs();
              if Args[1] = 'child' then
                File.WriteAllText(Args[3] + '.argument', Args[4]);
                File.WriteAllText(Args[3] + '.environment', S.Environment.GetEnvironmentVariable('FPAS_PROCESS_TEST_VALUE') ?? '');
                File.WriteAllText(Args[3] + '.tmp', S.Environment.ProcessId.ToString());
                File.Move(Args[3] + '.tmp', Args[3]);
                while true do Thread.Sleep(20); end while;
              else
                const Start: P.ProcessStartInfo := new P.ProcessStartInfo(Args[2]);
                Start.UseShellExecute := false;
                Start.CreateNoWindow := true;
                Start.ArgumentList.Add(Args[0]);
                Start.ArgumentList.Add('child');
                Start.ArgumentList.Add(Args[2]);
                Start.ArgumentList.Add(Args[3]);
                Start.ArgumentList.Add(Args[4]);
                discard P.Process.Start(Start);
                while not File.Exists(Args[3]) do Thread.Sleep(20); end while;
              end if;
            end;
            end program;
            """);
        var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source,
            "--output", assembly, "--reference-dir", TestEnvironment.ReferenceDirectory]);
        Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
        var ready = fixture.File("Grüße child.pid");
        const string value = "Grüße \"quoted\" \\";
        var host = Environment.GetEnvironmentVariable("DOTNET_HOST_PATH")
            ?? Path.Combine(Environment.GetEnvironmentVariable("DOTNET_ROOT")!, OperatingSystem.IsWindows() ? "dotnet.exe" : "dotnet");
        await using var owned = OwnedProcess.Start(host, [assembly, "parent", host, ready, value], fixture.Path,
            new Dictionary<string, string> { ["FPAS_PROCESS_TEST_VALUE"] = value });
        using var deadline = new CancellationTokenSource(TimeSpan.FromSeconds(10));
        await owned.WaitForExit(deadline.Token);
        Assert.Equal(0, owned.ExitCode);
        using var child = Process.GetProcessById(int.Parse(await File.ReadAllTextAsync(ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture));
        Assert.False(child.HasExited);
        Assert.Equal(value, await File.ReadAllTextAsync(ready + ".argument", deadline.Token));
        Assert.Equal(value, await File.ReadAllTextAsync(ready + ".environment", deadline.Token));
        try
        {
            await Task.WhenAll(owned.DisposeAsync().AsTask(), owned.DisposeAsync().AsTask());
            Assert.True(child.HasExited);
            if (OperatingSystem.IsLinux()) Assert.False(Directory.Exists("/proc/" + child.Id));
        }
        finally { await child.WaitForExitAsync(deadline.Token); }
    }
}
