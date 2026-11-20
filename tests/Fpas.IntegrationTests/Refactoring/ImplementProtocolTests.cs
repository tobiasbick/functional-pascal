// [Documentation](../../../docs/pascal/tools/refactoring.md#implement-a-protocol)
using System.Text.Json;
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Exercises compiler-checked protocol implementation plans.</summary>
/// <param name="sdk">SDK package used to resolve the test project.</param>
public sealed class ImplementProtocolTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Resolves inherited generic members, keeps matching declarations, and applies reviewed forwarding and bodies.</summary>
    [Fact]
    public async Task Cli_plans_and_applies_inherited_protocol_implementations()
    {
        const string source = """
            unit Demo.Adapters;
            uses System as Sys;

            public type IBase<T> = protocol
              Label: string;
              function Read(Value: T): T;
            end protocol;

            public type IReader<T> = protocol extends IBase<T>, Sys.IDisposable
              function Echo<U>(Value: U): U;
              where U: class;
              function Choose(Value: string): string;
              function Choose(Value: int): int;
            end protocol;

            public type Reader = record
              public Label: string;

              private function ReadValue(Value: int): int;
              begin
                return Value + 1;
              end function;

              public procedure Dispose();
              begin
              end procedure;
            end record;

            public function Existing(Value: Reader): string;
            begin
              return Value.Label;
            end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var at = RenameFixture.Select(directory, source, "Reader = record");
        var path = directory.File(".temp-data/protocol.json");
        var options = new[] { "implement-protocol", "Game/Game.fpasproj", "--file", at.File,
            "--line", FormattableString.Invariant($"{at.Line}"), "--column", FormattableString.Invariant($"{at.Column}"),
            "--protocol", "IReader<int>", "--plan", path, "--report", "json" };
        var missing = await RenameFixture.Cli(directory, options);
        Assert.Equal(1, missing.ExitCode);
        using (var report = JsonDocument.Parse(missing.Output))
        {
            var error = report.RootElement.GetProperty("error");
            Assert.Equal("refactor_not_supported", error.GetProperty("code").GetString());
            var message = error.GetProperty("message").GetString();
            Assert.Contains("Read = public function Read(Value: int): int;", message);
            Assert.Contains("Choose#1 = public function Choose(Value: int): int;", message);
            Assert.Contains("Choose#2 = public function Choose(Value: string): string;", message);
            Assert.DoesNotContain("Dispose", message);
            Assert.DoesNotContain("Label", message);
        }
        Assert.False(File.Exists(path));
        Assert.Equal(source, File.ReadAllText(at.File));

        var preview = await RenameFixture.Cli(directory, [.. options, "--forward", "Read=ReadValue",
            "--member", "Echo=return Value;", "--member", "Choose#1=return Value + 1;", "--member", "Choose#2=return Value;"]);
        Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
        var plan = RefactorPlanJson.Read(File.ReadAllBytes(path));
        Assert.Equal("implement-protocol", plan.Kind);
        Assert.Equal("IReader<int>", plan.Protocol!.Type);
        Assert.Equal(4, plan.Protocol.Members.Count);
        Assert.True(plan.ChangesPublicApi);
        Assert.Equal(source, File.ReadAllText(at.File));

        var applied = await RenameFixture.Cli(directory, "apply", "Game/Game.fpasproj", "--plan", path, "--no-restore");
        Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
        var result = File.ReadAllText(at.File);
        Assert.Contains("type Reader = record implements IReader<int>", result);
        Assert.Contains("return self.ReadValue(Value);", result);
        Assert.Contains("where U: class;", result);
        Assert.Contains("return Value.Label;", result);
    }

    /// <summary>Checks getter-only properties, source comments, and an inherited conformance no-op.</summary>
    [Fact]
    public async Task Adds_an_explicit_property_and_reuses_the_existing_implementation()
    {
        var source = """
            unit Demo.Adapters;
            uses System as Sys;
            type IBase = protocol
              Label: string;
            end protocol;
            type INamed = protocol extends IBase
            end protocol;
            type Reader = record implements Sys.IDisposable // retain header comment
              public procedure Dispose();
              begin
              end procedure;
            end record;
            end unit;
            """.ReplaceLineEndings("\n");
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var at = RenameFixture.Select(directory, source, "Reader = record");
        var request = new RefactoringRequest("implement-protocol", at, at.Line, at.Column,
            Protocol: new("INamed", [new("Label", "'Ready\n  now'")]));
        var prepared = CheckedRefactoring.Prepare(analysis, request);
        var document = Assert.Single(prepared.Edits.Documents);
        Assert.Contains("record implements Sys.IDisposable, INamed // retain header comment", document.NewText);
        Assert.Contains("  public Label: string := 'Ready\n  now';", document.NewText);
        File.WriteAllText(at.File, document.NewText);
        var updated = await RenameFixture.Analyze(directory);
        var already = CheckedRefactoring.Prepare(updated, request with { Protocol = new("IBase", []) });
        Assert.Equal(0, already.Edits.EditCount);
        Assert.Contains("already implements", Assert.Single(already.Notes));
    }

    /// <summary>Spells foreign signature types and preserves argument modes and line endings in a generated adapter.</summary>
    [Fact]
    public async Task Spells_a_signature_with_existing_imports_and_forwards_a_managed_argument()
    {
        const string contracts = """
            unit Demo.Contracts;
            uses System.Text as Text;
            public type IAppend = protocol
              procedure Append(ref Builder: Text.StringBuilder);
            end protocol;
            end unit;
            """;
        var source = """
            unit Demo.Adapters;
            uses Demo.Contracts as Contracts, System as Sys;
            type Writer = record
              private procedure Write(ref Value: Sys.Text.StringBuilder);
              begin
                discard Value.Append('written');
              end procedure;
            end record;
            end unit;
            """.ReplaceLineEndings("\r\n");
        using var directory = RenameFixture.Create(sdk, source);
        File.WriteAllText(directory.File("Game/Contracts.fpas"), contracts);
        var analysis = await RenameFixture.Analyze(directory);
        var at = RenameFixture.Select(directory, source, "Writer = record");
        var prepared = CheckedRefactoring.Prepare(analysis, new("implement-protocol", at, at.Line, at.Column,
            Protocol: new("Contracts.IAppend", [new("Append", "Write", Forward: true)])));
        var result = Assert.Single(prepared.Edits.Documents).NewText;
        Assert.Contains("  public procedure Append(ref Builder: Sys.Text.StringBuilder);\r\n", result);
        Assert.Contains("    self.Write(ref Builder);\r\n", result);
        Assert.DoesNotContain('\n', result.Replace("\r\n", "", StringComparison.Ordinal));
    }

    /// <summary>Inserts a required type import using a free alias in the target unit.</summary>
    [Fact]
    public async Task Adds_an_import_for_a_type_used_only_by_the_protocol()
    {
        const string contracts = """
            unit Demo.Contracts;
            uses System.Text as Text;
            public type IWriter = protocol
              function Write(Value: Text.StringBuilder): Text.StringBuilder;
            end protocol;
            end unit;
            """;
        const string source = """
            unit Demo.Adapters;
            uses Demo.Contracts as Contracts, System.IO as Text;
            type Writer = record end record;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        File.WriteAllText(directory.File("Game/Contracts.fpas"), contracts);
        var analysis = await RenameFixture.Analyze(directory);
        var at = RenameFixture.Select(directory, source, "Writer = record");
        var prepared = CheckedRefactoring.Prepare(analysis, new("implement-protocol", at, at.Line, at.Column,
            Protocol: new("Contracts.IWriter", [new("Write", "return Value;")])));
        var result = Assert.Single(prepared.Edits.Documents).NewText;
        Assert.Contains("System.IO as Text, System.Text as Text2;", result);
        Assert.Contains("public function Write(Value: Text2.StringBuilder): Text2.StringBuilder;", result);
    }

    /// <summary>Coalesces inherited slots with different nullable guarantees into one compatible method.</summary>
    [Fact]
    public async Task Combines_nullable_requirements_of_shared_slots()
    {
        const string source = """
            unit Demo.Adapters;
            type IStrict = protocol
              function Read(Value: string): string;
            end protocol;
            type ILoose = protocol
              function Read(Value: string?): string?;
            end protocol;
            type IReader = protocol extends IStrict, ILoose
            end protocol;
            type Reader = record
            end record;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var at = RenameFixture.Select(directory, source, "Reader = record");
        var prepared = CheckedRefactoring.Prepare(analysis, new("implement-protocol", at, at.Line, at.Column,
            Protocol: new("IReader", [new("Read", "if Value = null then return 'empty'; end if; return Value;")])));
        var result = Assert.Single(prepared.Edits.Documents).NewText;
        Assert.Contains("public function Read(Value: string?): string;", result);
        Assert.DoesNotContain("Read#1", result);
    }

    /// <summary>Rejects invalid bodies, unexpected members and accidental changes to existing overload resolution.</summary>
    [Fact]
    public async Task Refuses_incompatible_or_unrelated_edits()
    {
        const string source = """
            unit Demo.Adapters;
            type IReader = protocol
              function Read(): int;
            end protocol;
            type Reader = record
              private function TextValue(): string;
              begin
                return 'text';
              end function;
            end record;
            function Identify(Value: object): int; begin return 1; end function;
            function Identify(Value: IReader): int; begin return 2; end function;
            function Run(Value: Reader): int; begin return Identify(Value); end function;
            end unit;
            """;
        using var directory = RenameFixture.Create(sdk, source);
        var analysis = await RenameFixture.Analyze(directory);
        var at = RenameFixture.Select(directory, source, "Reader = record");
        Assert.Contains("missing member", Fail("Unknown", "return 1;").Message);
        Assert.Contains("itself", Fail("Read", "Read", forward: true).Message);
        Assert.Equal("refactor_not_supported", Fail("Read", "return 1; end function; public function Extra(): int; begin return 2;").Code);
        Assert.Equal("refactor_conflict", Fail("Read", "TextValue", forward: true).Code);
        Assert.Equal("refactor_conflict", Fail("Read", "return 1;").Code);
        Assert.Equal(source, File.ReadAllText(at.File));

        ContextFailure Fail(string key, string value, bool forward = false)
            => Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(analysis,
                new("implement-protocol", at, at.Line, at.Column, Protocol: new("IReader", [new(key, value, forward)]))));
    }
}
