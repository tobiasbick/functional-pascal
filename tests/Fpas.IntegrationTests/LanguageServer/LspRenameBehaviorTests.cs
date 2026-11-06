using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using System.Text.Json;
using static Fpas.IntegrationTests.LanguageServer.LspFeatureTests;

namespace Fpas.IntegrationTests.LanguageServer;

public sealed class LspRenameBehaviorTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Applying_keyword_overload_capture_and_interface_renames_preserves_actual_program_output()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas");
        const string source = """
            program Example;
            uses System.Console as C;
            type IReader = interface function Read(): int; end interface;
            type Reader = record implements IReader
              Value: int;
              function Read(): int; begin return self.Value; end function;
            end record;
            // Transform and 😀 stay unchanged.
            function Transform(Value: int): int; begin return Value; end function;
            function Transform(Value: string): int; begin return Value.Length; end function;
            function Outer(): int;
            begin
              const Captured: int := 20;
              function Inner(): int; begin return Captured; end function;
              return Inner();
            end function;
            begin
              const R: Reader := new Reader(Value := 21);
              const I: IReader := R;
              const F: function(int): int := Transform;
              C.WriteLine(F(Transform(1)) + Transform('x') + Outer() + R.Read() + I.Read());
            end;
            end program;
            """;
        var text = source.Replace("\n", "\r\n", StringComparison.Ordinal);
        File.WriteAllText(directory.File("Game/Game.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Exe</OutputType></PropertyGroup></Project>");
        File.WriteAllText(path, text);
        var before = await BuildAndRun(directory); Assert.Equal("64", before.Output.Trim());
        var dll = directory.File("Game/bin/Debug/net10.0/Game.dll"); var bytes = File.ReadAllBytes(dll); var stamp = File.GetLastWriteTimeUtc(dll);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { });
        var version = 7; await client.Open(path, text, version);
        foreach (var (marker, name) in new[] { ("Transform(Value: int)", "type"), ("Captured: int", "SavedValue"), ("Read(): int; end interface", "Fetch") })
        {
            var changes = Result(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(text, marker), newName = name }));
            var document = Assert.Single(changes.GetProperty("documentChanges").EnumerateArray()); Assert.Equal(version, document.GetProperty("textDocument").GetProperty("version").GetInt32());
            text = Apply(text, document.GetProperty("edits"));
            await client.FullChange(path, text, ++version);
            Assert.Equal(source.Replace("\n", "\r\n", StringComparison.Ordinal), File.ReadAllText(path)); Assert.Equal(bytes, File.ReadAllBytes(dll)); Assert.Equal(stamp, File.GetLastWriteTimeUtc(dll));
        }
        Assert.Contains("function @type(Value: int)", text); Assert.Contains("function Transform(Value: string)", text);
        Assert.Contains("// Transform and 😀 stay unchanged.", text); Assert.DoesNotContain("Captured", text); Assert.DoesNotContain("Read()", text); Assert.Contains("I.Fetch()", text);
        Error(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(text, "SavedValue: int"), newName = "Inner" }), -32803);
        Error(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(text, "SavedValue: int"), newName = "é" }), -32602);
        await client.Stop(); File.WriteAllText(path, text);
        var after = await BuildAndRun(directory); Assert.Equal(before.Output, after.Output);
    }

    [Fact]
    public async Task Closed_source_edits_have_an_explicit_null_version_and_the_server_never_applies_them()
    {
        using var directory = sdk.CreateProject(); var path = directory.File("Game/Bridge.fpas"); var libraryPath = directory.File("Game/Library.fpas");
        const string library = "unit Library; public function Answer(): int; begin return 42; end function; end unit;";
        const string source = "unit Demo.Bridge; uses Library as L; public function Run(): int; begin return L.Answer(); end function; end unit;";
        File.WriteAllText(directory.File("Game/Game.fpasproj"), $"<Project Sdk=\"Fpas.NET.Sdk/{sdk.Version}\"><PropertyGroup><OutputType>Library</OutputType></PropertyGroup></Project>");
        File.WriteAllText(path, source); File.WriteAllText(libraryPath, library);
        var restore = await sdk.Run(directory, "restore", "Game/Game.fpasproj"); Assert.True(restore.ExitCode == 0, restore.Output + restore.Error);
        await using var client = new LspProbe(); await client.Initialize(directory.File("Game/Game.fpasproj")); await client.Notify("initialized", new { }); await client.Open(path, source, 9);
        var edit = Result(await client.Request("textDocument/rename", new { textDocument = Document(path), position = Position(source, "Answer();"), newName = "Fetch" }));
        var documents = edit.GetProperty("documentChanges").EnumerateArray().ToArray(); Assert.Equal(2, documents.Length);
        var closed = Assert.Single(documents, document => LspProbe.IsFileUri(document.GetProperty("textDocument").GetProperty("uri"), libraryPath));
        Assert.Equal(JsonValueKind.Null, closed.GetProperty("textDocument").GetProperty("version").ValueKind);
        Assert.Equal(9, Assert.Single(documents, document => LspProbe.IsFileUri(document.GetProperty("textDocument").GetProperty("uri"), path)).GetProperty("textDocument").GetProperty("version").GetInt32());
        Assert.Equal(source, File.ReadAllText(path)); Assert.Equal(library, File.ReadAllText(libraryPath)); await client.Stop();
    }

    private async Task<ProcessResult> BuildAndRun(TemporaryDirectory directory)
    {
        var built = await sdk.Run(directory, "build", "Game/Game.fpasproj", "--disable-build-servers"); Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var run = await ProcessRunner.RunAsync([directory.File("Game/bin/Debug/net10.0/Game.dll")]); Assert.True(run.ExitCode == 0, run.Output + run.Error); return run;
    }

    internal static string Apply(string original, JsonElement edits)
    {
        int Offset(JsonElement position)
        {
            var wanted = position.GetProperty("line").GetInt32(); var line = 0; var index = 0;
            while (line < wanted && index < original.Length)
            { if (original[index] == '\r') { index++; if (index < original.Length && original[index] == '\n') index++; line++; } else if (original[index++] == '\n') line++; }
            Assert.Equal(wanted, line); return index + position.GetProperty("character").GetInt32();
        }
        var changes = edits.EnumerateArray().Select(edit => (Start: Offset(edit.GetProperty("range").GetProperty("start")), End: Offset(edit.GetProperty("range").GetProperty("end")), Text: edit.GetProperty("newText").GetString()!)).OrderByDescending(edit => edit.Start).ToArray();
        var result = original;
        foreach (var edit in changes) result = result[..edit.Start] + edit.Text + result[edit.End..];
        return result;
    }
}
