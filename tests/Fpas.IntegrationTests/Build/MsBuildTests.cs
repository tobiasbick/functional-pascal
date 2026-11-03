using Fpas.Tests.Support;
using System.Xml.Linq;
using System.Numerics;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Mandelbrot_example_builds_and_matches_independently_calculated_pixels()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("Mandelbrot"));
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "mandelbrot");
        File.Copy(Path.Combine(fixture, "main.fpas"), project.File("Mandelbrot/main.fpas"));
        File.WriteAllText(project.File("Mandelbrot/Mandelbrot.fpasproj"),
            File.ReadAllText(Path.Combine(fixture, "Mandelbrot.fpasproj")).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Mandelbrot/Mandelbrot.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await sdk.Run(project, "Mandelbrot/bin/Debug/net10.0/Mandelbrot.dll");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var rows = result.Output.Split(Environment.NewLine);
        Assert.Equal(25, rows.Length);
        Assert.Equal("", rows[^1]);
        for (var y = 0; y < 24; y++)
        {
            Assert.Equal(80, rows[y].Length);
            for (var x = 0; x < 80; x++)
            {
                var c = new Complex(-2.0 + 3.0 * x / 79, -1.0 + 2.0 * y / 23);
                var z = Complex.Zero;
                var count = 0;
                while (count < 50 && z.Real * z.Real + z.Imaginary * z.Imaginary <= 4.0)
                {
                    z = z * z + c;
                    count++;
                }
                Assert.Equal(count == 50 ? '#' : ' ', rows[y][x]);
            }
        }
    }

    [Fact]
    public async Task Public_bool_and_double_signatures_are_consumable_from_CSharp()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            public function Scale(Value: double; Enabled: bool): double;
            begin
              if Enabled then return Value * 2.0; else return 0; end if;
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            if (Demo.Bridge.Scale(2.5, true) != 5.0 || Demo.Bridge.Scale(2.5, false) != 0.0)
                throw new InvalidOperationException("Incorrect FPAS signature or result.");
            Console.WriteLine("Typed interop");
            """);
        var build = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal("Typed interop" + Environment.NewLine, result.Output);
    }

    [Fact]
    public async Task Mixed_solution_builds_and_executes_with_transitive_references()
    {
        using var project = sdk.CreateProject();
        var result = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Contains("FPAS: compiling", result.Output);
        var executed = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("C#:FPAS" + Environment.NewLine, executed.Output);
        Assert.True(File.Exists(project.File("Host/bin/Debug/net10.0/Shared.dll")));
        Assert.True(File.Exists(project.File("Host/bin/Debug/net10.0/Game.dll")));
        Assert.Empty(Directory.GetFiles(project.File("Game/obj"), "*.cs", SearchOption.AllDirectories));
    }

    [Fact]
    public async Task Incremental_builds_track_source_content_and_missing_outputs()
    {
        using var project = sdk.CreateProject();
        var first = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        var unchanged = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(unchanged.ExitCode == 0, unchanged.Output + unchanged.Error);
        Assert.DoesNotContain("FPAS: compiling", unchanged.Output);
        var source = project.File("Game/Bridge.fpas");
        File.AppendAllText(source, "\n// Changed input content.\n");
        var changed = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(changed.ExitCode == 0, changed.Output + changed.Error);
        Assert.Contains("FPAS: compiling", changed.Output);
        File.Delete(project.File("Game/obj/Debug/net10.0/Game.dll"));
        var missing = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(missing.ExitCode == 0, missing.Output + missing.Error);
        Assert.Contains("FPAS: compiling", missing.Output);
    }

    [Fact]
    public async Task Diagnostics_fail_the_build_with_source_locations()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Message(Value: string): string; begin const X := 'bad'; return Value; end function; end unit;");
        var result = await sdk.Run(project, "build", "Game.slnx");
        Assert.NotEqual(0, result.ExitCode);
        Assert.Contains("FP2002", result.Output + result.Error);
        Assert.Contains("Bridge.fpas(1,", result.Output + result.Error);
        Assert.False(File.Exists(project.File("Host/bin/Debug/net10.0/Host.dll")));
    }

    [Fact]
    public async Task Clean_and_release_rebuild_use_the_same_project_contract()
    {
        using var project = sdk.CreateProject();
        var result = await sdk.Run(project, "build", "Game.slnx", "--configuration", "Release");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var clean = await sdk.Run(project, "clean", "Game.slnx", "--configuration", "Release");
        Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
        Assert.False(File.Exists(project.File("Game/bin/Release/net10.0/Game.dll")));
        Assert.False(File.Exists(project.File("Game/obj/Release/net10.0/fpas.compile.json")));
        var rebuild = await sdk.Run(project, "msbuild", "Game.slnx", "-restore", "-t:Rebuild", "-p:Configuration=Release");
        Assert.True(rebuild.ExitCode == 0, rebuild.Output + rebuild.Error);
        var executed = await sdk.Run(project, "Host/bin/Release/net10.0/Host.dll");
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("C#:FPAS" + Environment.NewLine, executed.Output);
    }

    [Fact]
    public async Task Package_references_flow_to_the_CSharp_host_at_runtime()
    {
        using var project = sdk.CreateProject();
        var version = "1.0.0-test.g" + Guid.NewGuid().ToString("N");
        var packed = await sdk.Run(project, "pack", "Shared/Shared.csproj", "--output", sdk.Feed,
            "-p:PackageId=Demo.Shared", "-p:PackageVersion=" + version);
        Assert.True(packed.ExitCode == 0, packed.Output + packed.Error);
        var path = project.File("Game/Game.fpasproj");
        var xml = XDocument.Load(path);
        xml.Descendants("ProjectReference").Single().ReplaceWith(new XElement("PackageReference",
            new XAttribute("Include", "Demo.Shared"), new XAttribute("Version", version)));
        xml.Save(path);
        var build = await sdk.Run(project, "build", "Host/Host.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var executed = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.Equal(0, executed.ExitCode);
        Assert.Equal("C#:FPAS" + Environment.NewLine, executed.Output);
    }

    [Fact]
    public async Task Changed_reference_metadata_invalidates_the_compiler_cache()
    {
        using var project = sdk.CreateProject();
        var first = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(first.ExitCode == 0, first.Output + first.Error);
        var source = project.File("Shared/SharedApi.cs");
        File.WriteAllText(source, File.ReadAllText(source).Replace("Decorate", "Changed", StringComparison.Ordinal));
        var result = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.NotEqual(0, result.ExitCode);
        Assert.Contains("FPAS: compiling", result.Output);
        Assert.Contains("FP3003", result.Output + result.Error);
    }

    [Fact]
    public async Task Compile_remove_items_and_source_additions_are_honored()
    {
        using var project = sdk.CreateProject();
        var extra = project.File("Game/Excluded.fpas");
        File.WriteAllText(extra, "not valid FPAS");
        var path = project.File("Game/Game.fpasproj");
        var xml = XDocument.Load(path);
        xml.Root!.Add(new XElement("ItemGroup", new XElement("Compile", new XAttribute("Remove", "Excluded.fpas"))));
        xml.Save(path);
        var excluded = await sdk.Run(project, "build", "Game.slnx");
        Assert.True(excluded.ExitCode == 0, excluded.Output + excluded.Error);
        xml.Descendants("Compile").Remove();
        xml.Save(path);
        var added = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.NotEqual(0, added.ExitCode);
        Assert.Contains("FP2001", added.Output + added.Error);
        File.Delete(extra);
        var removed = await sdk.Run(project, "build", "Game.slnx", "--no-restore");
        Assert.True(removed.ExitCode == 0, removed.Output + removed.Error);
    }
}
