using System.Reflection;
using System.Xml.Linq;

namespace Fpas.Cli.Projects;

internal static class TestTemplate
{
    private static string Version(string name) => typeof(TestTemplate).Assembly.GetCustomAttributes<AssemblyMetadataAttribute>()
        .Single(attribute => attribute.Key == "Fpas" + name + "Version").Value!;

    internal static string Project(string sdkVersion, string framework, string runner = "vstest")
    {
        var project = XElement.Parse(ProjectFiles.Project(sdkVersion, library: true));
        project.Element("PropertyGroup")!.Add(new XElement("IsTestProject", "true"), new XElement("IsPackable", "false"),
            new XElement("ManagePackageVersionsCentrally", "false"));
        if (runner == "mtp" && framework != "mstest") throw new ArgumentException("The FPAS MTP template uses --framework mstest; other MTP hosts may be supplied as existing projects.");
        if (runner == "mtp")
        {
            project.Element("PropertyGroup")!.Element("OutputType")!.Value = "Exe";
            project.Element("PropertyGroup")!.Add(new XElement("EnableMSTestRunner", "true"));
        }
        var packages = new XElement("ItemGroup");
        if (runner == "vstest") packages.Add(Package("Microsoft.NET.Test.Sdk", "TestSdk", adapter: true));
        else packages.Add(Package("Microsoft.Testing.Extensions.TrxReport", "TestingPlatform", adapter: true));
        packages.Add(framework switch
        {
            "xunit" => new[] { Package("xunit", "Xunit"), Package("xunit.runner.visualstudio", "XunitAdapter", adapter: true) },
            "mstest" => new[] { Package("MSTest.TestFramework", "MSTest"), Package("MSTest.TestAdapter", "MSTest", adapter: true) },
            "nunit" => new[] { Package("NUnit", "NUnit"), Package("NUnit3TestAdapter", "NUnitAdapter", adapter: true) },
            _ => throw new ArgumentException("--framework must be xunit, mstest or nunit.")
        });
        project.Add(packages);
        return project + Environment.NewLine;
    }

    private static XElement Package(string name, string version, bool adapter = false)
        => new("PackageReference", new XAttribute("Include", name), new XAttribute("Version", Version(version)),
            adapter ? new XAttribute("PrivateAssets", "all") : null);

    internal static string Source(string identifier, string framework) => framework switch
    {
        "xunit" => $$"""
            unit {{identifier}};

            uses Xunit as X;

            [X.FactAttribute]
            public procedure Addition();
            begin
              X.Assert.Equal<int>(4, 2 + 2);
            end procedure;
            end unit;

            """,
        "mstest" => $$"""
            unit {{identifier}};

            uses Microsoft.VisualStudio.TestTools.UnitTesting as T;

            [T.TestClassAttribute]
            public type Tests = record
              [T.TestMethodAttribute]
              procedure Addition();
              begin
                T.Assert.AreEqual<int>(4, 2 + 2);
              end procedure;
            end record;
            end unit;

            """,
        "nunit" => $$"""
            unit {{identifier}};

            uses NUnit.Framework as N, NUnit.Framework.Legacy as L;

            [N.TestFixtureAttribute]
            public type Tests = record
              [N.TestAttribute]
              procedure Addition();
              begin
                L.ClassicAssert.AreEqual(4, 2 + 2);
              end procedure;
            end record;
            end unit;

            """,
        _ => throw new ArgumentException("--framework must be xunit, mstest or nunit.")
    };

    internal static string EntryPoint(string identifier) => $$"""
        async program {{identifier}}.Runner;

        uses
          System as Sys, System.Reflection as Reflection, System.Collections.Generic as Collections,
          Microsoft.Testing.Platform.Builder as Platform,
          Microsoft.VisualStudio.TestTools.UnitTesting.TestApplicationBuilderExtensions as MSTest,
          Microsoft.Testing.Extensions.TrxReportExtensions as Trx;

        begin
          const Args: string[] := Sys.Environment.GetCommandLineArgs();
          const Arguments: Collections.List<string> := new Collections.List<string>();
          for Index: int := 1 to Args.Length - 1 do Arguments.Add(Args[Index]); end for;
          const Builder: Platform.ITestApplicationBuilder := await Platform.TestApplication.CreateBuilderAsync(Arguments.ToArray());
          MSTest.AddMSTest(Builder, function(): Collections.IEnumerable<Reflection.Assembly> begin
            return [Reflection.Assembly.GetExecutingAssembly()];
          end function);
          Trx.AddTrxReportProvider(Builder);
          using const Application: Platform.ITestApplication := await Builder.BuildAsync();
          Sys.Environment.ExitCode := await Application.RunAsync();
        end;
        end program;

        """;
}
