using Fpas.IntegrationTests.Build;
using Fpas.Tests.Support;
using Fpas.Workspaces.Analysis;
using Fpas.Workspaces.Loading;
using System.Text.Json;
using System.Xml.Linq;

namespace Fpas.IntegrationTests.ApiDocumentation;

internal static class DocumentationFixture
{
    internal static TemporaryDirectory Create(SdkPackageFixture sdk)
    {
        var directory = sdk.CreateProject();
        var project = XDocument.Load(directory.File("Shared/Shared.csproj"));
        project.Root!.Add(new XElement("PropertyGroup", new XElement("GenerateDocumentationFile", "true"), new XElement("AllowUnsafeBlocks", "true"), new XElement("NoWarn", "1591")));
        project.Save(directory.File("Shared/Shared.csproj"));
        File.WriteAllText(directory.File("Shared/DocumentationApi.cs"), """
            namespace Lookup;
            /// <summary>API with &lt;XML&gt; text, <see cref="System.String"/> and <see langword="null"/>.</summary>
            public class Api<T> where T : class, new()
            {
                static Api() => System.IO.File.WriteAllText("initializer-executed", "bad");
                /// <summary>Construct an API.</summary>
                public Api() { }
                /// <summary>Echo text.</summary><param name="value">Input text.</param><returns>Echoed text.</returns>
                public static string Echo(string value = "default") => value;
                /// <summary>Echo generic values.</summary><typeparam name="U">Result type.</typeparam>
                public U Echo<U>(ref T value, out T previous, in U other) { previous = value; return other; }
                /// <summary>Describe array shapes.</summary>
                public static void Shapes(int[,][] grid, string?[]? text, double? number, params object[] more) { }
                /// <summary>Describe unmanaged signatures without executing them.</summary>
                public static unsafe void Unmanaged(int* value, delegate*<int, string> callback) { }
                /// <summary>Indexed value.</summary>
                public string this[int index] { get => throw new System.Exception("Getter executed"); set => throw new System.Exception("Setter executed"); }
                /// <summary>A field.</summary>
                public const int Limit = 42;
                /// <summary>An event.</summary>
                public event System.EventHandler? Changed { add { } remove { } }
                /// <summary>A conversion.</summary>
                public static implicit operator int(Api<T> value) => 42;
                /// <summary>Nested generic type.</summary>
                public class Nested<U> where U : struct
                {
                    /// <summary>Nested generic parameter positions.</summary>
                    public V Combine<V>(T left, U right, V value, Api<T>.Nested<int> nested) => value;
                }
                private static void Hidden() { }
                public int ReadOnly { get => throw new System.Exception("Getter executed"); private set { } }
            }
            internal class PrivateOwner { public class PublicNested { public void HiddenByOwner() { } } }
            public class SecondApi { public static string Echo(string value) => value; }
            """.Replace("\"initializer-executed\"", JsonSerializer.Serialize(directory.File("initializer-executed")), StringComparison.Ordinal));
        File.WriteAllText(directory.File("Game/Bridge.fpas"), """
            // Source documentation container.
            unit Demo.Bridge;
            uses Demo.SharedApi as Shared;
            // Echoes the provided source value.
            //
            // ## Parameters
            // - `Value`: Text to decorate.
            public function Message(Value: string): string;
            begin return Shared.Decorate(Value); end function;
            // Source point.
            public type Point = record
              // Horizontal position.
              public X: int;
              // Returns the coordinate.
              public function Read(): int; begin return self.X; end function;
            end record;
            end unit;
            """);
        return directory;
    }

    internal static async Task<WorkspaceAnalysis> Analyze(TemporaryDirectory directory)
    {
        using var deadline = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        var context = await WorkspaceLoader.LoadAsync(new(directory.File("Game/Game.fpasproj")), deadline.Token);
        var analysis = WorkspaceAnalysis.Analyze(context, deadline.Token);
        Assert.True(analysis.Success, string.Join('\n', context.Issues.Select(issue => issue.Message)) + string.Join('\n', analysis.Projects.SelectMany(project => project.Result?.Diagnostics ?? [])));
        return analysis;
    }
    internal static Task<ProcessResult> Cli(TemporaryDirectory directory, params string[] args)
        => ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "doc", .. args], workingDirectory: directory.Path, timeLimit: TimeSpan.FromMinutes(3));
}
