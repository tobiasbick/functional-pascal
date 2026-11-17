using System.Text.Json;
using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Unsupported_CLR_signatures_explain_the_feature_and_preserve_supported_overloads()
    {
        using var project = sdk.CreateProject();
        var path = project.File("Shared/Shared.csproj");
        var definition = XDocument.Load(path);
        definition.Root!.Add(new XElement("PropertyGroup", new XElement("AllowUnsafeBlocks", "true")));
        definition.Save(path);
        File.WriteAllText(project.File("Shared/Limits.cs"), """
            namespace Shared;
            public delegate ref int RefHandler();
            public readonly struct DefaultValue
            {
                public DefaultValue(Span<int> values) => Value = values.Length;
                public int Value { get; }
            }
            public struct Pixel { public byte Red; public byte Green; }
            public struct Named { public string Name; }
            public unsafe sealed class Limits
            {
                private static int value;
                public static ref int ReadRef() => ref value;
                public static ref int OnlyRef() => ref value;
                public static int ReadRef(int number) => number;
                public static Span<int> ReadSpan() => new int[1];
                public static void UsePointer(int* pointer) { }
                public static void Varargs(__arglist) { }
                public static delegate*<void> Callback => null;
                public static int* Pointer;
                public int Number { get; init; }
                public ref int this[int index] => ref value;
                public static int Choose(int number) => number;
                public static int Choose(Span<int> numbers) => numbers.Length;
                public static int Count<T>(T[] values) where T : unmanaged => values.Length;
            }
            """);
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        (string Body, string Feature)[] cases =
        [
            ("discard L.ReadRef();", "ref returns"),
            ("discard L.OnlyRef;", "ref returns"),
            ("const Handler: function(): int := L.ReadSpan;", "byref-like type 'System.Span`1'"),
            ("discard L.ReadSpan();", "byref-like type 'System.Span`1'"),
            ("L.UsePointer(null);", "pointers"),
            ("L.Varargs();", "calling conventions"),
            ("discard L.Callback;", "function pointers"),
            ("discard L.Pointer;", "pointers"),
            ("discard L.Callback.ToString();", "function pointers"),
            ("const V: L := new L(); V.Number := 1;", "init-only setters"),
            ("const V: L := new L(); discard V[0];", "ref returns"),
            ("procedure Test(Handler: S.RefHandler); begin discard Handler(); end procedure;", "ref returns")
        ];
        foreach (var (body, feature) in cases)
        {
            var source = "program Invalid; uses Shared.Limits as L, Shared as S; begin " + body + " end; end program;";
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], source);
            Assert.NotEqual(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.Equal("FP3005", diagnostic.GetProperty("code").GetString());
            Assert.Contains(feature, diagnostic.GetProperty("message").GetString());
            Assert.True(diagnostic.GetProperty("location").GetProperty("column").GetInt32() > 1);
            Assert.False(File.Exists(project.File("Invalid.dll")));
        }
        foreach (var body in new[] { "discard L.Count<string>(['a']);", "discard L.Count([new S.Named()]);" })
        {
            var source = "program Invalid; uses Shared.Limits as L, Shared as S; begin " + body + " end; end program;";
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], source);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.Equal("FP3018", diagnostic.GetProperty("code").GetString());
            Assert.Contains("CLR constraints", diagnostic.GetProperty("message").GetString());
        }
        var run = await RunInterop(project, """
            program Supported;
            uses Shared.Limits as L, Shared as S, System.Console as Console;
            begin
              Console.WriteLine(L.Choose(42)); Console.WriteLine(new S.DefaultValue().Value);
              Console.WriteLine(L.Choose([1, 2, 3])); Console.WriteLine(new S.DefaultValue([4, 5]).Value);
              Console.WriteLine(L.Count<int>([1, 2])); Console.WriteLine(L.Count([new S.Pixel()]));
            end;
            end program;
            """);
        Assert.Equal(0, run.ExitCode);
        Assert.Equal(string.Join(Environment.NewLine, "42", "0", "3", "2", "2", "1") + Environment.NewLine, run.Output);
    }
}
