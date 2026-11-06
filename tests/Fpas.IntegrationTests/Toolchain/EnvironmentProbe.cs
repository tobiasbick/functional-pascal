using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Toolchain;

internal static class EnvironmentProbe
{
    internal static async Task<string> Build(TemporaryDirectory root, SdkPackageFixture sdk)
    {
        File.WriteAllText(root.File("Probe.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType><UseAppHost>true</UseAppHost></PropertyGroup>
            </Project>
            """);
        File.WriteAllText(root.File("main.fpas"), """
            program Probe;

            uses System.Environment as Env, System.IO.File as File, System.Threading.Thread as Thread;

            begin
              if args.Length <> 1 or args[0] <> '--version' then return; end if;
              const Ready: string := Env.GetEnvironmentVariable('FPAS_ENV_PROBE_READY') ?? 'ready.pid';
              File.WriteAllText(Ready + '.tmp', Env.ProcessId.ToString());
              File.Move(Ready + '.tmp', Ready);

              while true do
                Thread.Sleep(20);
              end while;
            end;
            end program;
            """);
        new XDocument(new XElement("configuration", new XElement("packageSources", new XElement("clear"),
            new XElement("add", new XAttribute("key", "fpas"), new XAttribute("value", sdk.Feed)))))
            .Save(root.File("NuGet.Config"));
        var build = await ProcessRunner.RunAsync(["build", root.File("Probe.fpasproj"), "--verbosity", "quiet"],
            workingDirectory: root.Path, timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var path = root.File("bin/Debug/net10.0/Probe" + (OperatingSystem.IsWindows() ? ".exe" : ""));
        Assert.True(File.Exists(path), "The SDK did not produce the version-probe fixture apphost.");
        return path;
    }
}
