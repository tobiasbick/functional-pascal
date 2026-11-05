using Fpas.IntegrationTests.Build;
using Fpas.IntegrationTests.Projects;
using Fpas.Tests.Support;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.Testing;

internal static class TestProjectFixture
{
    internal static async Task<string> Create(TemporaryDirectory directory, SdkPackageFixture sdk, string framework, bool detailed = false, string name = "Tests")
    {
        var result = await ProjectCliFixture.Cli(directory, "init", "test", name, "--framework", framework,
            "--sdk-version", sdk.Version, "--sdk-feed", sdk.Feed);
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var project = directory.File(name + "/" + name + ".fpasproj");
        if (detailed)
        {
            File.Copy(Path.Combine(AppContext.BaseDirectory, "Fixtures", "testing", framework + ".fpas"), directory.File(name + "/" + name + ".fpas"), overwrite: true);
            if (framework == "xunit")
            {
                var xml = XDocument.Load(project);
                xml.Root!.Add(new XElement("ItemGroup", new XElement("Reference", new XAttribute("Include", "Fpas.Compiler"),
                    new XAttribute("HintPath", typeof(Compiler.FpasCompiler).Assembly.Location))));
                xml.Save(project);
            }
        }
        return project;
    }
}
