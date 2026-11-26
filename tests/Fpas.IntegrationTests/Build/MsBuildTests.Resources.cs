// [Documentation](../../../docs/build/msbuild.md#embedded-resources-and-localization)
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Resources_follow_ui_culture_from_build_and_publish_and_refresh_on_edit()
    {
        using var project = sdk.CreateProject();
        const string projectPath = "Game/Game.fpasproj";
        File.WriteAllText(project.File(projectPath), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup>
                <TargetFramework>net10.0</TargetFramework>
                <OutputType>Exe</OutputType>
                <RootNamespace>Localized</RootNamespace>
              </PropertyGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            program Localized;
            uses System.Console as C; uses System.Globalization as G; uses System.Reflection as Refl; uses System.Resources as R;
            begin
              G.CultureInfo.CurrentUICulture := G.CultureInfo.GetCultureInfo(args[0]);
              const Strings: R.ResourceManager := new R.ResourceManager('Localized.Text.Strings', Refl.Assembly.GetExecutingAssembly());
              C.WriteLine(Strings.GetString('Greeting') ?? 'missing greeting');
              C.WriteLine(Strings.GetString('NeutralOnly') ?? 'missing fallback');
            end;
            end program;
            """);
        Directory.CreateDirectory(project.File("Game/Text"));
        var neutral = project.File("Game/Text/Strings.resx");
        var german = project.File("Game/Text/Strings.de.resx");
        WriteResx(neutral, ("Greeting", "Hello"), ("NeutralOnly", "English fallback"));
        WriteResx(german, ("Greeting", "Grüße"));

        var check = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check",
            project.File(projectPath), "--report", "json"], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(check.ExitCode == 0, check.Output + check.Error);
        Assert.Empty(Directory.GetFiles(project.File("Game"), "*.resources", SearchOption.AllDirectories));
        Assert.Empty(Directory.GetFiles(project.File("Game"), "*.resources.dll", SearchOption.AllDirectories));

        async Task<ProcessResult> Build()
        {
            var result = await sdk.Run(project, "build", projectPath, "--no-restore");
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            return result;
        }

        async Task Verify(string directory, string culture, string greeting)
        {
            Assert.True(File.Exists(Path.Combine(directory, "de", "Game.resources.dll")));
            var result = await ProcessRunner.RunAsync([Path.Combine(directory, "Game.dll"), culture], workingDirectory: directory);
            Assert.True(result.ExitCode == 0, result.Output + result.Error);
            Assert.Equal(greeting + Environment.NewLine + "English fallback" + Environment.NewLine, result.Output);
        }

        await Build();
        var output = project.File("Game/bin/Debug/net10.0");
        await Verify(output, "de", "Grüße");
        await Verify(output, "de-AT", "Grüße");
        await Verify(output, "fr-FR", "Hello");
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);

        WriteResx(neutral, ("Greeting", "Welcome"), ("NeutralOnly", "English fallback"));
        Assert.Contains("FPAS: compiling", (await Build()).Output);
        await Verify(output, "fr-FR", "Welcome");
        WriteResx(german, ("Greeting", "Willkommen"));
        Assert.DoesNotContain("FPAS: compiling", (await Build()).Output);
        await Verify(output, "de", "Willkommen");

        var published = project.File("published");
        var publish = await sdk.Run(project, "publish", projectPath, "--no-restore", "--output", published);
        Assert.True(publish.ExitCode == 0, publish.Output + publish.Error);
        await Verify(published, "de-AT", "Willkommen");
        await Verify(published, "fr-FR", "Welcome");
        var clean = await sdk.Run(project, "clean", projectPath);
        Assert.True(clean.ExitCode == 0, clean.Output + clean.Error);
        Assert.False(File.Exists(Path.Combine(output, "de", "Game.resources.dll")));
        Assert.Empty(Directory.GetFiles(project.File("Game/obj/Debug"), "*.resources", SearchOption.AllDirectories));
    }

    [Fact]
    public async Task Explicit_resource_names_and_binary_inputs_work_in_referenced_projects()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Game.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup>
                <TargetFramework>net10.0</TargetFramework>
                <OutputType>Library</OutputType>
              </PropertyGroup>
              <ItemGroup>
                <EmbeddedResource Update="Text.resx" LogicalName="Sample.Labels.resources" />
                <EmbeddedResource Update="Text.de.resx" LogicalName="Sample.Labels.de.resources" />
                <EmbeddedResource Include="payload, sample.bin" LogicalName="Sample.Payload" />
              </ItemGroup>
            </Project>
            """);
        WriteResx(project.File("Game/Text.resx"), ("Text", "Neutral"));
        WriteResx(project.File("Game/Text.de.resx"), ("Text", "Deutsch"));
        File.WriteAllBytes(project.File("Game/payload, sample.bin"), [42, 17]);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Localized;
            uses System.Globalization as G; uses System.IO as IO; uses System.Reflection as Refl; uses System.Resources as R;
            public function Label(Culture: string): string;
            begin
              const Strings: R.ResourceManager := new R.ResourceManager('Sample.Labels', Refl.Assembly.GetExecutingAssembly());
              return Strings.GetString('Text', G.CultureInfo.GetCultureInfo(Culture)) ?? 'missing';
            end function;
            public function ByteValue(): int;
            begin
              const Resource: IO.Stream? := Refl.Assembly.GetExecutingAssembly().GetManifestResourceStream('Sample.Payload');
              if Resource = null then return -1; end if;
              using const Input: IO.Stream := Resource;
              return Input.ReadByte();
            end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        const string consumerProject = "Consumer/Consumer.fpasproj";
        File.WriteAllText(project.File(consumerProject), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Exe</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Main.fpas"), """
            program Consumer;
            uses System.Console as C; uses Localized as L;
            begin
              C.WriteLine(L.Label('de-DE'));
              C.WriteLine(L.Label('fr-FR'));
              C.WriteLine(L.ByteValue());
            end;
            end program;
            """);
        var check = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "check",
            project.File(consumerProject), "--report", "json"], timeLimit: TimeSpan.FromMinutes(2));
        Assert.True(check.ExitCode == 0, check.Output + check.Error);
        var result = await sdk.Run(project, "build", consumerProject, "--no-restore");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        var run = await ProcessRunner.RunAsync([project.File("Consumer/bin/Debug/net10.0/Consumer.dll")]);
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "Deutsch", "Neutral", "42", ""), run.Output);
    }

    private static void WriteResx(string path, params (string Name, string Text)[] entries)
        => new XDocument(new XElement("root",
            new XElement("resheader", new XAttribute("name", "resmimetype"), new XElement("value", "text/microsoft-resx")),
            new XElement("resheader", new XAttribute("name", "version"), new XElement("value", "2.0")),
            entries.Select(entry => new XElement("data", new XAttribute("name", entry.Name),
                new XAttribute(XNamespace.Xml + "space", "preserve"), new XElement("value", entry.Text))))).Save(path);
}
