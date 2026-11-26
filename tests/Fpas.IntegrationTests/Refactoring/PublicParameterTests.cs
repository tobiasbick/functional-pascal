// [Documentation](../../../docs/pascal/tools/refactoring.md#public-signatures-across-projects)
using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Loading;
using Fpas.Workspaces.Refactoring;
using System.Globalization;
using System.Text.Json;

namespace Fpas.IntegrationTests.Refactoring;

/// <summary>Exercises checked parameter edits across source-owned assemblies.</summary>
/// <param name="sdk">The matching SDK package for consumer projects.</param>
public sealed class PublicParameterTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    /// <summary>Previews, applies and runs all three parameter operations across a project reference.</summary>
    [Fact]
    public async Task Public_parameter_plans_update_a_separate_consumer_and_preserve_an_overload()
    {
        const string library = """
            unit Demo.Bridge;
            public function Evaluate(Used: int; Ignored: int): bool; begin return Used > 0; end function;
            public function Evaluate(Text: string): bool; begin return Text <> ''; end function;
            end unit;
            """;
        const string caller = """
            program Consumer;
            uses Demo.Bridge as B; uses System.Console as Console;
            begin
              var Number: long := 41;
              Console.WriteLine(B.Evaluate(Ignored := 99, Used := 41) and B.Evaluate('other'));
            end;
            end program;
            """;
        using var directory = RenameFixture.Create(sdk, library);
        const string project = "Consumer/Consumer.fpasproj";
        const string callerFile = "Consumer/Main.fpas";
        Directory.CreateDirectory(directory.File("Consumer"));
        File.WriteAllText(directory.File(project), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Exe</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(directory.File(callerFile), caller);
        var libraryPath = directory.File("Game/Bridge.fpas");

        var addArguments = new[] { "--name", "Extra", "--type", "int", "--argument", "5" };
        var rejected = await Preview("add-parameter", callerFile, "Evaluate(Ignored", addArguments);
        Assert.Equal(1, rejected.ExitCode);
        using (var report = JsonDocument.Parse(rejected.Output))
            Assert.Contains("--closed-context", report.RootElement.GetProperty("error").GetProperty("message").GetString());
        Assert.False(File.Exists(directory.File(".temp-data/add-parameter.json")));
        Assert.Equal(library, File.ReadAllText(libraryPath));
        Assert.Equal(caller, File.ReadAllText(directory.File(callerFile)));

        await Apply("add-parameter", callerFile, "Evaluate(Ignored", addArguments);
        Assert.Contains("Evaluate(Used: int; Ignored: int; Extra: int)", File.ReadAllText(libraryPath));
        Assert.Contains("Evaluate(Ignored := 99, Used := 41, Extra := 5)", File.ReadAllText(directory.File(callerFile)));

        await Apply("remove-parameter", callerFile, "Ignored :=", []);
        Assert.Contains("Evaluate(Used: int; Extra: int)", File.ReadAllText(libraryPath));
        Assert.Contains("Evaluate(Used := 41, Extra := 5)", File.ReadAllText(directory.File(callerFile)));

        var call = RenameFixture.Select(directory, File.ReadAllText(directory.File(callerFile)), "Evaluate(Used", callerFile);
        var plan = await Apply("change-signature", "Game/Bridge.fpas", "Used: int", ["--type", "long", "--mode", "in",
            "--argument", $"{callerFile}:{Number(call.Line)}:{Number(call.Column)}=Number"]);
        Assert.Equal(directory.File(callerFile), Assert.Single(plan.SignatureChange!.Arguments).File);
        Assert.Contains("Evaluate(in Used: long; Extra: int)", File.ReadAllText(libraryPath));
        Assert.Contains("Evaluate(Used := in Number, Extra := 5) and B.Evaluate('other')", File.ReadAllText(directory.File(callerFile)));

        var built = await sdk.Run(directory, "build", project, "--no-restore", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        var result = await ProcessRunner.RunAsync([directory.File("Consumer/bin/Debug/net10.0/Consumer.dll")]);
        Assert.True(result.ExitCode == 0, result.Error);
        Assert.Equal("True", result.Output.Trim());

        async Task<RefactorPlan> Apply(string operation, string file, string marker, string[] options)
        {
            var preview = await Preview(operation, file, marker, [.. options, "--closed-context"]);
            Assert.True(preview.ExitCode == 0, preview.Output + preview.Error);
            var planPath = directory.File(".temp-data/" + operation + ".json");
            var prepared = RefactorPlanJson.Read(File.ReadAllBytes(planPath));
            Assert.True(prepared.ClosedContext);
            Assert.True(prepared.ChangesPublicApi);
            Assert.Equal(2, prepared.Files.Count);
            var applied = await RenameFixture.Cli(directory, "apply", project, "--plan", planPath, "--no-restore", "--report", "json");
            Assert.True(applied.ExitCode == 0, applied.Output + applied.Error);
            using var report = JsonDocument.Parse(applied.Output);
            Assert.True(report.RootElement.GetProperty("rechecked").GetBoolean());
            return prepared;
        }

        Task<ProcessResult> Preview(string operation, string file, string marker, string[] options)
        {
            var at = RenameFixture.Select(directory, File.ReadAllText(directory.File(file)), marker, file);
            return RenameFixture.Cli(directory, [operation, project, "--file", file, "--line", Number(at.Line),
                "--column", Number(at.Column), "--plan", directory.File(".temp-data/" + operation + ".json"),
                "--report", "json", .. options]);
        }

        static string Number(int value) => value.ToString(CultureInfo.InvariantCulture);
    }

    /// <summary>Refuses a known foreign project consumer and the same consumer referenced only as a binary.</summary>
    [Fact]
    public async Task Closed_context_refuses_non_editable_consumers()
    {
        const string library = "unit Demo.Bridge; public function Message(Value: string): string; begin return Value; end function; end unit;";
        using var directory = RenameFixture.Create(sdk, library);
        var at = RenameFixture.Select(directory, library, "Value: string");
        var request = new RefactoringRequest("change-signature", at, at.Line, at.Column,
            SignatureChange: new(null, "ref", []), ClosedContext: true);
        var solution = await RenameFixture.Analyze(directory, "Game.slnx");
        var foreign = Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(solution, request));
        Assert.Contains("non-editable project consumer", foreign.Message);
        Assert.Contains("Host.csproj", foreign.Message);

        var built = await sdk.Run(directory, "build", "Host/Host.csproj", "-m:1", "--disable-build-servers");
        Assert.True(built.ExitCode == 0, built.Output + built.Error);
        Directory.CreateDirectory(directory.File("Client"));
        File.WriteAllText(directory.File("Client/Main.fpas"), "unit Client; end unit;");
        File.WriteAllText(directory.File("Client/Client.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup>
                <ProjectReference Include="../Game/Game.fpasproj" />
                <Reference Include="Host"><HintPath>../Host/bin/Debug/net10.0/Host.dll</HintPath></Reference>
              </ItemGroup>
            </Project>
            """);
        var client = await RenameFixture.Analyze(directory, "Client/Client.fpasproj");
        var binary = Assert.Throws<ContextFailure>(() => CheckedRefactoring.Prepare(client, request));
        Assert.Contains("non-editable binary boundary", binary.Message);
        Assert.Contains("Host.dll", binary.Message);
        Assert.Equal(library, File.ReadAllText(at.File));
    }
}
