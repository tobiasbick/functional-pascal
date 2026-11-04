using Fpas.Tests.Support;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces.Support;

internal sealed class WaitingBuild : IDisposable
{
    private readonly TemporaryDirectory directory = new();
    internal string Ready => directory.File("ready.pid");

    internal static async Task<WaitingBuild> Prepare(string project)
    {
        var helper = new WaitingBuild();
        try
        {
            var source = helper.directory.File("Wait.fpas"); var assembly = helper.directory.File("Wait.dll");
            File.WriteAllText(source, """
                program Wait;
                uses System as S, System.IO.File as File, System.Threading.Thread as Thread;
                begin
                  const Ready: string := S.Environment.GetCommandLineArgs()[1];
                  File.WriteAllText(Ready + '.tmp', S.Environment.ProcessId.ToString());
                  File.Move(Ready + '.tmp', Ready);
                  while true do Thread.Sleep(20); end while;
                end;
                end program;
                """);
            var compiled = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", source, "--output", assembly, "--reference-dir", TestEnvironment.ReferenceDirectory]);
            Assert.True(compiled.ExitCode == 0, compiled.Output + compiled.Error);
            var xml = XDocument.Load(project);
            xml.Root!.Add(new XElement("Target", new XAttribute("Name", "WaitForCancellation"), new XAttribute("BeforeTargets", "FpasPrepareCompilation"),
                new XElement("Exec", new XAttribute("Command", $"\"{TestEnvironment.DotnetHost}\" \"{assembly}\" \"{helper.Ready}\""))));
            xml.Save(project); return helper;
        }
        catch { helper.Dispose(); throw; }
    }

    public void Dispose() => directory.Dispose();
}
