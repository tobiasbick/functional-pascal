using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Build.Support;
using Fpas.IntegrationTests.Processes.Support;
using Fpas.Tests.Support;
using System.Diagnostics;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisSignalTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Console_interrupt_returns_exit_130_and_one_cancelled_JSON_document_after_joining_build_tasks()
    {
        foreach (var arguments in new[]
        {
            new[] { "check", "Game/Game.fpasproj", "--diagnostics", "json" },
            new[] { "check", "Game/Game.fpasproj", "--fix", "--diagnostics", "json" },
            new[] { "doc", "Game/Game.fpasproj", "--name", "System.Console", "--report", "json" },
            new[] { "refactor", "rename", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "17", "--new-name", "Describe", "--plan", ".temp-data/rename.json", "--report", "json" },
            new[] { "refactor", "apply", "Game/Game.fpasproj", "--plan", ".temp-data/rename.json", "--report", "json" },
            new[] { "query", "refs", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" },
            new[] { "query", "callees", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" },
            new[] { "query", "callers", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas", "--line", "6", "--column", "25" }
        })
        {
            using var project = sdk.CreateProject();
            var afterFix = arguments.Contains("--fix");
            var afterRename = arguments is ["refactor", "apply", ..];
            if (afterFix) File.WriteAllText(project.File("Game/Bridge.fpas"), "unit Demo.Bridge; public function Read(Value: integer): int; begin return Value; end function; end unit;");
            using var waiting = await WaitingBuild.Prepare(project.File("Game/Game.fpasproj"));
            if (afterFix || afterRename)
            {
                var xml = XDocument.Load(project.File("Game/Game.fpasproj"));
                xml.Root!.Elements("Target").Single(target => (string?)target.Attribute("Name") == "WaitForCancellation")
                    .SetAttributeValue("Condition", "$([System.IO.File]::ReadAllText('$(MSBuildProjectDirectory)/Bridge.fpas').Contains('" + (afterFix ? "integer" : "Message") + "')) != 'True'");
                xml.Save(project.File("Game/Game.fpasproj"));
            }
            if (afterRename)
            {
                var planned = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "refactor", "rename", "Game/Game.fpasproj", "--file", "Game/Bridge.fpas",
                    "--line", "6", "--column", "17", "--new-name", "Describe", "--plan", ".temp-data/rename.json", "--report", "json"],
                    workingDirectory: project.Path, timeLimit: TimeSpan.FromMinutes(2));
                Assert.True(planned.ExitCode == 0, planned.Output + planned.Error);
            }
            using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
            await using var process = InterruptibleProcess.Start(TestEnvironment.DotnetHost, [TestEnvironment.Metadata("FpasCliPath"), .. arguments], project.Path);
            process.Input.Dispose();
            using var stdout = new StreamReader(process.Output); using var stderr = new StreamReader(process.Error);
            var output = stdout.ReadToEndAsync(deadline.Token); var error = stderr.ReadToEndAsync(deadline.Token);
            int? child = null;
            try
            {
                while (!File.Exists(waiting.Ready))
                {
                    if (process.HasExited) Assert.Fail("CLI exited before the preparation task became ready: " + await output + await error);
                    await Task.Delay(20, deadline.Token);
                }
                child = int.Parse(await File.ReadAllTextAsync(waiting.Ready, deadline.Token), System.Globalization.CultureInfo.InvariantCulture);
                await process.Interrupt();
                await process.WaitForExit(deadline.Token);
                Assert.Equal(130, process.ExitCode); Assert.Equal("", await error);
                using var json = JsonDocument.Parse(await output);
                Assert.Equal(afterRename ? "applied" : "cancelled", json.RootElement.GetProperty("status").GetString());
                Assert.Equal("cancelled", json.RootElement.GetProperty("error").GetProperty("code").GetString());
                if (afterFix)
                {
                    var fixes = json.RootElement.GetProperty("fixes");
                    Assert.Equal("applied", fixes.GetProperty("state").GetString()); Assert.False(fixes.GetProperty("rechecked").GetBoolean());
                    Assert.All(fixes.GetProperty("files").EnumerateArray(), file => Assert.True(file.GetProperty("written").GetBoolean()));
                    Assert.Empty(json.RootElement.GetProperty("diagnostics").EnumerateArray());
                    Assert.DoesNotContain("integer", File.ReadAllText(project.File("Game/Bridge.fpas")));
                    Assert.Empty(Directory.GetFiles(project.File("Game"), ".fpas-edit-*"));
                }
                if (afterRename)
                {
                    Assert.False(json.RootElement.GetProperty("rechecked").GetBoolean());
                    Assert.All(json.RootElement.GetProperty("files").EnumerateArray(), file => Assert.True(file.GetProperty("written").GetBoolean()));
                    Assert.Empty(json.RootElement.GetProperty("diagnostics").EnumerateArray());
                    Assert.Contains("Describe", File.ReadAllText(project.File("Game/Bridge.fpas")));
                    Assert.Empty(Directory.GetFiles(project.File("Game"), ".fpas-edit-*"));
                }
                Assert.Throws<ArgumentException>(() => Process.GetProcessById(child.Value));
            }
            finally
            {
                await process.TerminateAsync();
            }
        }
    }
}
