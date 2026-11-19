// [Documentation](../../../docs/pascal/tools/refactoring.md#change-a-parameter-type-or-mode)
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using System.Globalization;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Exercises checked changes to private FPAS signatures.</summary>
/// <param name="sdk">SDK package used to resolve the test project.</param>
public sealed class ChangeSignatureTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Checks CLI preview, required replacements, and application by exact call position.</summary>
    [Fact]
    public async Task Cli_applies_a_mode_change_and_distinct_replacements_by_call_position()
    {
        const string source = """
            unit Demo.Bridge;
            function Accept(Value: int; Limit: int): bool; begin return Value > Limit; end function;
            public function Run(): bool;
            begin
              var First: int := 7;
              var Second: int := 9;
              discard Accept(First, 1);
              discard Accept(3, 2);
              return Accept(Limit := 1, Value := 5);
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var at = RenameFixture.Select(directory, source, "Value: int");
        var first = RenameFixture.Select(directory, source, "Accept(3, 2)");
        var second = RenameFixture.Select(directory, source, "Accept(Limit");
        var path = directory.File(".temp-data/signature.json");
        var options = new[] { "change-signature", "Game/Game.fpasproj", "--file", at.File,
            "--line", Text(at.Line), "--column", Text(at.Column), "--mode", "ref", "--plan", path, "--report", "json" };
        var missing = await RenameFixture.Cli(directory, options);
        Assert.Equal(1, missing.ExitCode);
        using (var report = JsonDocument.Parse(missing.Output))
            Assert.Equal("refactor_conflict", report.RootElement.GetProperty("error").GetProperty("code").GetString());
        Assert.False(File.Exists(path));
        Assert.Equal(source, File.ReadAllText(at.File));

        var preview = await RenameFixture.Cli(directory, [.. options,
            "--argument", $"{Text(first.Line)}:{Text(first.Column)}=First",
            "--argument", $"{Text(second.Line)}:{Text(second.Column)}=Second"]);
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("change-signature", plan.Kind);
        Assert.Equal("ref", plan.SignatureChange!.Mode);
        Assert.Equal(new[] { "First", "Second" }, plan.SignatureChange.Arguments.Select(argument => argument.Expression));
        Assert.Equal(source, File.ReadAllText(at.File));

        var applied = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        Assert.Equal(source.Replace("Value: int", "ref Value: int")
            .Replace("Accept(First, 1)", "Accept(ref First, 1)")
            .Replace("Accept(3, 2)", "Accept(ref First, 2)")
            .Replace("Value := 5", "Value := ref Second"), File.ReadAllText(at.File));

        static string Text(int value) => value.ToString(CultureInfo.InvariantCulture);
    }

    /// <summary>Checks valid modes, preserved defaults, and refusal of incompatible bodies or overload changes.</summary>
    [Fact]
    public async Task Overlay_preserves_untouched_calls_and_checks_type_mode_and_constructor_overloads()
    {
        const string source = """
            unit Demo.Bridge;
            uses System.Text as Txt;
            function Accept(Count: int := 1): bool; begin return Count > 0; end function;
            function Capacity(Size: int): bool; begin discard new Txt.StringBuilder(Size); return true; end function;
            function Read(ref Input: int): bool; begin return Input > 0; end function;
            procedure Fill(ref Output: int); begin Output := 4; end procedure;
            public function Run(): bool;
            begin
              var Number: int := 2;
              var Wide: long := 3;
              Fill(ref Number);
              return Accept() and Read(ref Number) and Capacity(2);
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var wide = Prepare("Count: int", new("long", null, [Replacement("Accept()", "8")]));
        Assert.Equal(source.Replace("Count: int", "Count: long").Replace("Accept()", "Accept(Count := 8)"),
            Assert.Single(wide.Edits.Documents).NewText);
        var value = Prepare("Input: int", new(null, "value", []));
        Assert.Equal(source.Replace("Read(ref Input", "Read(Input").Replace("Read(ref Number)", "Read(Number)"),
            Assert.Single(value.Edits.Documents).NewText);
        var write = Prepare("Output: int", new(null, "out", []));
        Assert.Equal(source.Replace("ref Output", "out Output").Replace("Fill(ref Number)", "Fill(out Number)"),
            Assert.Single(write.Edits.Documents).NewText);
        var combined = Prepare("Output: int", new("long", "out", [Replacement("Fill(ref Number)", "Wide")]));
        Assert.Equal(source.Replace("ref Output: int", "out Output: long").Replace("Fill(ref Number)", "Fill(out Wide)"),
            Assert.Single(combined.Edits.Documents).NewText);
        var readOnly = Prepare("Input: int", new(null, "in", []));
        Assert.Equal(source.Replace("Read(ref Input", "Read(in Input").Replace("Read(ref Number)", "Read(in Number)"),
            Assert.Single(readOnly.Edits.Documents).NewText);
        var invalidBody = Assert.Throws<ContextFailure>(() => Prepare("Input: int", new(null, "out", [])));
        Assert.Equal("refactor_conflict", invalidBody.Code);
        var invalidWrite = Assert.Throws<ContextFailure>(() => Prepare("Output: int", new(null, "in", [])));
        Assert.Equal("refactor_conflict", invalidWrite.Code);
        var changedConstructor = Assert.Throws<ContextFailure>(() =>
            Prepare("Size: int", new("string", null, [Replacement("Capacity(2)", "'initial'")])));
        Assert.Equal("refactor_conflict", changedConstructor.Code);
        Assert.Contains("constructor", changedConstructor.Message, StringComparison.OrdinalIgnoreCase);

        PreparedRefactoring Prepare(string marker, RefactorSignatureChange change)
        {
            var at = RenameFixture.Select(directory, source, marker);
            return CheckedRefactoring.Prepare(analysis, new("change-signature", at, at.Line, at.Column, SignatureChange: change));
        }

        RefactorArgumentOverride Replacement(string marker, string expression)
        {
            var call = RenameFixture.Select(directory, source, marker);
            return new(new(call.Line, call.Column), expression);
        }
    }
}
