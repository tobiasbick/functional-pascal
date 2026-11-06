using System.Text.Json;
using System.Xml.Linq;
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Projects;

public sealed partial class PublishingTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Theory]
    [InlineData("Debug", "aot")]
    [InlineData("Release", "aot")]
    [InlineData("Debug", "single-file")]
    [InlineData("Release", "single-file")]
    [InlineData("Debug", "self-contained")]
    [InlineData("Release", "self-contained")]
    [InlineData("Debug", "apphost")]
    [InlineData("Release", "apphost")]
    [InlineData("Debug", "framework-dependent")]
    [InlineData("Release", "framework-dependent")]
    public async Task Published_mixed_program_runs_without_sources_feed_or_compiler(string configuration, string mode)
    {
        using var directory = new TemporaryDirectory();
        var project = await Create(directory);
        var destination = directory.File("published");
        var result = await Cli(directory, "publish", project, "--output", destination, "--configuration", configuration, "--mode", mode, "--report", "json");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        using var json = JsonDocument.Parse(result.Output);
        Assert.True(json.RootElement.GetProperty("success").GetBoolean());
        Assert.Equal(mode, json.RootElement.GetProperty("publish").GetProperty("mode").GetString());
        Assert.NotEmpty(json.RootElement.GetProperty("publish").GetProperty("files").EnumerateArray());
        Assert.DoesNotContain(Directory.GetFiles(destination, "*", SearchOption.AllDirectories), path => Path.GetExtension(path) is ".snk" or ".cs" or ".fpas");
        var delivery = directory.File("delivery"); Directory.CreateDirectory(delivery);
        var name = OperatingSystem.IsWindows() ? "App.exe" : "App";
        if (mode is "aot" or "single-file") File.Copy(Path.Combine(destination, name), Path.Combine(delivery, name));
        else foreach (var file in Directory.GetFiles(destination)) File.Copy(file, Path.Combine(delivery, Path.GetFileName(file)));
        if (!OperatingSystem.IsWindows() && mode != "framework-dependent") File.SetUnixFileMode(Path.Combine(delivery, name), UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute);
        Directory.Move(directory.File("App"), directory.File("App.hidden"));
        Directory.Move(directory.File("Shared"), directory.File("Shared.hidden"));
        var environment = mode is "aot" or "single-file" or "self-contained" ? new Dictionary<string, string?>
        { ["PATH"] = "", ["DOTNET_ROOT"] = directory.File("absent"), ["DOTNET_ROOT_X64"] = directory.File("absent"), ["DOTNET_HOST_PATH"] = directory.File("absent"),
            ["DOTNET_BUNDLE_EXTRACT_BASE_DIR"] = directory.File("bundle") } : null;
        var ran = mode == "framework-dependent"
            ? await ProcessRunner.RunAsync([Path.Combine(delivery, "App.dll")], workingDirectory: delivery)
            : await ProcessRunner.RunExecutableAsync(Path.Combine(delivery, name), [], workingDirectory: delivery, environment: environment);
        Assert.True(ran.ExitCode == 0, ran.Output + ran.Error); Assert.Empty(ran.Error);
        Assert.Equal("published:42\n", ran.Output.ReplaceLineEndings("\n"));
    }

    private async Task<string> Create(TemporaryDirectory directory)
    {
        var project = await ProjectCliFixture.Init(directory, sdk);
        Directory.CreateDirectory(directory.File("Shared"));
        File.WriteAllText(directory.File("Shared/Shared.csproj"), "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net10.0</TargetFramework></PropertyGroup></Project>");
        File.WriteAllText(directory.File("Shared/Value.cs"), "namespace Shared; public static class Value { public static int Read() => 21; }");
        var xml = XDocument.Load(project); xml.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Shared/Shared.csproj")))); xml.Save(project);
        File.WriteAllText(directory.File("App/main.fpas"), """
            async program App;
            uses System as Sys, System.Console as Console, System.Threading.Tasks as Tasks, Shared.Value as Foreign,
              Fpas.Core.RecordAttribute as RecordMarker;
            type Id = distinct int;
            type Box<T> = record
              Value: T;
            end record;
            async function Twice(Value: int): Tasks.Task<int>;
            requires Value >= 0;
            ensures (Result: int): Result = Value * 2;
            begin
              await Tasks.Task.Delay(1);
              const Map: Sys.Func<int, int> := function(Item: int): int => Item + Value;
              return Map.Invoke(Value);
            end function;
            begin
              const Wrapped: Box<Id> := new Box<Id>(new Id(Foreign.Read()));
              const Value: int := await Twice(Wrapped.Value.Value);
              if Value <> 42 then raise new Sys.InvalidOperationException('Incorrect publishing result'); end if;
              if not Sys.Attribute.IsDefined(typeof(Box<Id>), typeof(RecordMarker)) then
                raise new Sys.InvalidOperationException('Missing Core record metadata');
              end if;
              Console.WriteLine('published:' + Value.ToString());
            end;
            end program;
            """);
        return project;
    }

    private static Task<ProcessResult> Cli(TemporaryDirectory directory, params string[] args) =>
        ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), .. args], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(8));
}
