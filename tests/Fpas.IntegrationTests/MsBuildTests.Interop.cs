using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Global_namespace_types_round_trip_through_two_Fpas_libraries()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses GlobalValue as Value;
            public function Echo(Item: Value): Value;
            begin return Item; end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Middle"));
        File.WriteAllText(project.File("Middle/Middle.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Middle/Api.fpas"), """
            unit Demo.Middle;
            uses Demo.Bridge as Bridge, GlobalValue as Value;
            public function Read(): int;
            begin
              const Item: Value := new Value();
              Item.Number := 23;
              return Bridge.Echo(Item).Number;
            end function;
            end unit;
            """);
        var host = project.File("Host/Host.csproj");
        File.WriteAllText(host, File.ReadAllText(host).Replace("../Game/Game.fpasproj", "../Middle/Middle.fpasproj", StringComparison.Ordinal));
        File.WriteAllText(project.File("Host/Program.cs"), "Console.WriteLine(Demo.Middle.Read());");
        var build = await sdk.Run(project, "build", "Host/Host.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("23" + Environment.NewLine, run.Output);
    }

    [Fact]
    public async Task StreamReader_example_builds_and_reads_the_copied_file()
    {
        using var project = sdk.CreateProject();
        var fixture = Path.Combine(AppContext.BaseDirectory, "Fixtures", "read-file");
        Directory.CreateDirectory(project.File("Reader"));
        foreach (var file in Directory.GetFiles(fixture))
            File.WriteAllText(project.File("Reader/" + Path.GetFileName(file)), File.ReadAllText(file).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Reader/ReadFile.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await sdk.Run(project, "Reader/bin/Debug/net10.0/ReadFile.dll");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal(File.ReadAllText(Path.Combine(fixture, "sample.txt")) + "Read attempt finished" + Environment.NewLine, result.Output);
    }

    [Fact]
    public async Task Nullable_and_imported_type_signatures_are_consumed_by_CSharp()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System.Text as Text;
            public function Echo(Value: string?): string?;
            begin return Value; end function;
            public function MaybeNumber(Present: bool): int?;
            begin return if Present then 7 else null end if; end function;
            public function Builder(Value: Text.StringBuilder): Text.StringBuilder;
            begin return Value; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            string? absent = Demo.Bridge.Echo(null);
            if (absent is not null || Demo.Bridge.MaybeNumber(false).HasValue || Demo.Bridge.MaybeNumber(true) != 7)
                throw new Exception("Invalid nullable metadata or result.");
            var builder = new System.Text.StringBuilder("CLR");
            if (!ReferenceEquals(builder, Demo.Bridge.Builder(builder))) throw new Exception("Object identity changed.");
            Console.WriteLine("Nullable interop");
            """);
        var build = await sdk.Run(project, "build", "Game.slnx", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Nullable interop" + Environment.NewLine, run.Output);
    }

    [Theory]
    [InlineData("", "begin using const A: Resource := new Resource('A'); using const B: Resource := new Resource('B'); Resource.Note('body'); end; Console.WriteLine(Resource.Log);",
        "open:A;open:B;body;close:B;close:A;")]
    [InlineData("function Work(): int; begin using const A: Resource := new Resource('A'); return 42; end function;",
        "Console.WriteLine(Work()); Console.WriteLine(Resource.Log);", "42\nopen:A;close:A;")]
    [InlineData("", "for I: int := 1 to 3 do using const R: Resource := new Resource('loop'); if I = 1 then continue; end if; if I = 2 then break; end if; end for; Console.WriteLine(Resource.Log);",
        "open:loop;close:loop;open:loop;close:loop;")]
    [InlineData("", "try using const A: Resource := new Resource('A'); using const B: Resource := new Resource('bad-open'); catch E: Sys.InvalidOperationException do Resource.Note('caught'); finally Resource.Note('finally'); end try; Console.WriteLine(Resource.Log);",
        "open:A;open:bad-open;close:A;caught;finally;")]
    [InlineData("", "try using const A: Resource := new Resource('A'); using const B: Resource := new Resource('bad-close'); Resource.Note('body'); catch E: Sys.InvalidOperationException do Resource.Note('caught'); end try; Console.WriteLine(Resource.Log);",
        "open:A;open:bad-close;body;close:bad-close;close:A;caught;")]
    [InlineData("", "try using const R: Resource := new Resource('A'); raise new Sys.InvalidOperationException('x'); catch E: Sys.Exception when Resource.Filter('first') do Resource.Note('caught'); finally Resource.Note('finally'); end try; Console.WriteLine(Resource.Log);",
        "open:A;filter:first;close:A;caught;finally;")]
    [InlineData("", "try raise new Sys.InvalidOperationException('x'); catch E: Sys.Exception when Resource.ExplodingFilter() do Resource.Note('wrong'); catch E: Sys.Exception do Resource.Note('fallback'); end try; Console.WriteLine(Resource.Log);",
        "filter-throw;fallback;")]
    [InlineData("", "using const R: Resource? := Resource.Maybe(false); Console.WriteLine('null resource');", "null resource")]
    [InlineData("", "var C: Demo.Counter := new Demo.Counter(); C.Increment(); const Copy: Demo.Counter := C; Copy.Increment(); Console.WriteLine(C.Value); Console.WriteLine(Copy.Value); begin using const Owned: Demo.Counter := C; end; Console.WriteLine(Resource.Log);",
        "1\n1\ncounter:1;")]
    [InlineData("", "begin using const Owned: Demo.Counter? := Resource.MaybeCounter(true); end; Console.WriteLine(Resource.Log);", "counter:4;")]
    [InlineData("", "const R: Resource := new Resource('N'); R.Optional := null; R.OptionalField := null; const Text: string? := R.Optional; Console.WriteLine(Text ?? 'property'); Console.WriteLine(R.OptionalField ?? 'field');", "property\nfield")]
    [InlineData("", "const Text: string? := Resource.MaybeContract(); Console.WriteLine(Text ?? 'contract'); Resource.Accept(null); Console.WriteLine(Resource.Log);", "contract\nnull;")]
    [InlineData("", "try using const R: Resource := new Resource('bad-close'); raise new Sys.Exception('original'); catch Failure: Sys.Exception do Console.WriteLine(Failure.Message); end try; Console.WriteLine(Resource.Log);", "cleanup\nopen:bad-close;close:bad-close;")]
    [InlineData("", "const R: Resource := new Demo.DerivedResource('D'); Console.WriteLine(R.Label()); R.Number := 7; R.Field := 8; Resource.StaticField := 9; Console.WriteLine(R.Next().Number + R.Field + Resource.StaticField); Console.WriteLine(Resource.Overload(R)); Console.WriteLine(Resource.Overload(null)); begin using const Interface: Sys.IDisposable := R; end; Console.WriteLine(Resource.Log);",
        "derived:D\n24\nresource\nresource\nopen:D;close:D;")]
    public async Task Interop_and_resource_lifetimes_match_CLR_semantics(string declarations, string body, string expected)
    {
        using var project = sdk.CreateProject();
        var result = await RunInterop(project, "program App; uses Demo as Demo, Demo.Resource as Resource, System as Sys, System.Console as Console; " + declarations +
            " begin Resource.Reset(); " + body + " end; end program;");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal(expected.Replace("\n", Environment.NewLine, StringComparison.Ordinal) + Environment.NewLine, result.Output);
    }

    [Fact]
    public async Task Rethrow_preserves_the_original_CLR_stack()
    {
        using var project = sdk.CreateProject();
        var result = await RunInterop(project, """
            program Rethrow;
            uses Demo.Resource as Resource, System as Sys, System.Console as Console;
            procedure Wrap();
            begin try Resource.ThrowOriginal(); catch Failure: Sys.Exception do raise; end try; end procedure;
            begin
              try Wrap(); catch Failure: Sys.Exception do
                Console.WriteLine(Failure.Message);
                Console.WriteLine((Failure.StackTrace ?? '').Contains('ThrowOriginal'));
              end try;
            end;
            end program;
            """);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal("original" + Environment.NewLine + "True" + Environment.NewLine, result.Output);
    }

    [Theory]
    [InlineData("const R: Resource := new Resource('N'); if R.Optional <> null then Console.WriteLine(R.Optional.Length); end if;")]
    [InlineData("const R: Resource? := Resource.Maybe(false); discard Resource.Require(R);")]
    public async Task Nullable_metadata_does_not_allow_unsafe_dereferences_or_arguments(string body)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), "program Unsafe; uses Demo.Resource as Resource, System.Console as Console; begin " + body + " end; end program;");
        var path = project.File("Game/Game.fpasproj");
        File.WriteAllText(path, File.ReadAllText(path).Replace("<OutputType>Library</OutputType>", "<OutputType>Exe</OutputType>", StringComparison.Ordinal));
        var result = await sdk.Run(project, "build", "Game/Game.fpasproj");
        Assert.NotEqual(0, result.ExitCode);
        Assert.Contains("FP3013", result.Output + result.Error);
    }

    private async Task<ProcessResult> RunInterop(TemporaryDirectory project, string source)
    {
        File.WriteAllText(project.File("Game/Bridge.fpas"), source);
        var path = project.File("Game/Game.fpasproj");
        File.WriteAllText(path, File.ReadAllText(path).Replace("<OutputType>Library</OutputType>", "<OutputType>Exe</OutputType>", StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Game/Game.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        return await sdk.Run(project, "Game/bin/Debug/net10.0/Game.dll");
    }
}
