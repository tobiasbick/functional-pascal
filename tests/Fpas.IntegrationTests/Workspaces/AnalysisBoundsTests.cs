using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Loading;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Workspaces;

public sealed class AnalysisBoundsTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Excessive_MSBuild_output_is_drained_bounded_and_explicitly_rejected()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Game/Game.fpasproj"); var xml = XDocument.Load(path);
        xml.Root!.Add(new XElement("Target", new XAttribute("Name", "ExcessiveOutput"), new XAttribute("BeforeTargets", "FpasDescribeAnalysisContext"),
            new XElement("Warning", new XAttribute("Code", "OUTPUT_BOUND"), new XAttribute("Text", "$([System.String]::new('x', 1100000))"))));
        xml.Save(path);
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(path), deadline.Token);
        Assert.Empty(context.Projects);
        var issue = Assert.Single(context.Issues);
        Assert.True(issue.OutputTruncated);
        Assert.InRange(System.Text.Encoding.UTF8.GetByteCount(issue.Message), 1, 1_049_000);
        Assert.False(File.Exists(project.File("Game/bin/Debug/net10.0/Game.dll")));
    }
}
