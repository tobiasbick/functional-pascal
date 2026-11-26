// [Documentation](../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Analysis;
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;
using System.Text.Json;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class UsageTemplateTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Cli_selects_the_discovered_overload_and_checks_cleanup_in_the_consumer_context()
    {
        using var directory = DocumentationFixture.Create(sdk);
        var project = directory.File("Game/Game.fpasproj");
        var lookup = await DocumentationFixture.Cli(directory, project, "--name", "System.IO.File.OpenRead", "--report", "json");
        Assert.True(lookup.ExitCode == 0, lookup.Output + lookup.Error);
        using var declarations = JsonDocument.Parse(lookup.Output);
        var entry = Assert.Single(declarations.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        var id = entry.GetProperty("documentationId").GetString()!;
        Assert.False(entry.TryGetProperty("usage", out _));
        var response = await DocumentationFixture.Cli(directory, project, "--usage", id, "--report", "json", "--no-restore");
        Assert.True(response.ExitCode == 0, response.Output + response.Error);
        using var report = JsonDocument.Parse(response.Output);
        var selected = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray());
        Assert.Equal(id, selected.GetProperty("documentationId").GetString());
        var usage = selected.GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        Assert.Equal("Path", Assert.Single(usage.GetProperty("inputs").EnumerateArray()).GetString());
        Assert.Contains("using const Resource: IO.FileStream := IO.File.OpenRead(Path);", usage.GetProperty("source").GetString());
        Assert.False(File.Exists(directory.File("Game/bin/Debug/net10.0/Game.dll")));
    }

    [Fact]
    public async Task Usage_respects_modes_nullable_types_async_results_and_the_actual_overload()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Usage.cs"), """
            #nullable enable
            namespace Sample;
            public static class Actions
            {
                public static int NoInputs() => throw new System.Exception();
                public static void Modes(ref int first, out string? text, in double number) => throw new System.Exception();
                public static System.Threading.Tasks.Task<string?> Read(string? name) => throw new System.Exception();
                public static System.Threading.Tasks.ValueTask<System.IAsyncDisposable> Lease() => throw new System.Exception();
                public static System.Threading.Tasks.Task? Pending() => null;
                public static System.Threading.Tasks.Task<System.Threading.Tasks.Task> NestedTask() => throw new System.Exception();
                public static void OnlySpan(System.ReadOnlySpan<byte> data) => throw new System.Exception();
                public static void Choose(System.ReadOnlySpan<byte> data) => throw new System.Exception();
                public static void Choose(byte[] data) => throw new System.Exception();
                public static unsafe void Pointer(int* value) => throw new System.Exception();
                public static T Identity<T>(T x) => x;
            }
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        ApiUsage Select(string id)
        {
            var entry = Assert.Single(query.Search(new("M:Sample.Actions." + id, Assembly: "Shared", Usage: true)).Entries);
            return Assert.IsType<ApiUsage>(entry.Usage);
        }

        var example = Select("NoInputs");
        Assert.Equal("example", example.Status);
        Assert.Empty(example.Inputs);
        Assert.Contains("return Sample.Actions.NoInputs();", example.Source);
        var modes = Select("Modes(System.Int32@,System.String@,System.Double@)");
        Assert.Equal("template", modes.Status);
        Assert.Contains("ref First: int; out Text: string?; in Number: double", modes.Source);
        Assert.Contains("Modes(ref First, out Text, in Number)", modes.Source);
        Assert.Contains("return await Sample.Actions.Read(Name);", Select("Read(System.String)").Source);
        Assert.Contains("await using const Resource: Sys.IAsyncDisposable := await Sample.Actions.Lease();", Select("Lease").Source);
        Assert.Contains("null awaitable", Select("Pending").Reason);
        Assert.Contains("return await Sample.Actions.NestedTask();", Select("NestedTask").Source);
        Assert.Contains("Data: array of byte", Select("OnlySpan(System.ReadOnlySpan{System.Byte})").Source);
        Assert.NotNull(Select("Choose(System.Byte[])").Source);
        var wrongOverload = Select("Choose(System.ReadOnlySpan{System.Byte})");
        Assert.Equal("unavailable", wrongOverload.Status);
        Assert.Null(wrongOverload.Source);
        Assert.Contains("overload", wrongOverload.Reason);
        Assert.Contains("pointers", Select("Pointer(System.Int32*)").Reason);
        Assert.Contains("type arguments", Select("Identity``1(``0)").Reason);
    }

    [Fact]
    public async Task Constructors_and_instance_methods_bind_to_the_selected_definition()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Widget.cs"), """
            namespace Sample;
            public class Widget
            {
                public Widget() => throw new System.Exception();
                public Widget(int capacity) => throw new System.Exception();
                public virtual string Read(string receiver) => throw new System.Exception();
                public virtual int Read(int receiver) => throw new System.Exception();
                public int Size => throw new System.Exception();
            }
            public interface IMeter { int Read(); }
            public struct Counter
            {
                public int Count;
                public void Advance() => Count++;
                public System.Threading.Tasks.Task<int> AdvanceAsync() => System.Threading.Tasks.Task.FromResult(++Count);
            }
            public sealed class Lease : System.IDisposable { public void Dispose() { } }
            public abstract class Base { public Base() { } }
            public delegate int Transform(int value);
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        DocumentationEntry Select(string id) =>
            Assert.Single(query.Search(new("M:Sample." + id, Assembly: "Shared", Usage: true)).Entries);
        ApiUsage Usage(string id) => Assert.IsType<ApiUsage>(Select(id).Usage);

        var ctor = Select("Widget.#ctor(System.Int32)");
        Assert.Equal("constructor", ctor.Kind);
        Assert.Equal("Sample.Widget.ctor", ctor.QualifiedName);
        Assert.Contains("return new Sample.Widget(Capacity);", ctor.Usage!.Source);
        Assert.Equal("example", Usage("Widget.#ctor").Status);
        Assert.Contains("using const Resource: Sample.Lease := new Sample.Lease();", Usage("Lease.#ctor").Source);
        var read = Usage("Widget.Read(System.String)");
        Assert.Equal(["Receiver", "Receiver2"], read.Inputs);
        Assert.Contains("Use(Receiver: Sample.Widget; Receiver2: string): string;", read.Source);
        Assert.Contains("return Receiver.Read(Receiver2);", read.Source);
        Assert.Contains("Receiver2: int): int;", Usage("Widget.Read(System.Int32)").Source);
        Assert.Contains("return Receiver.Read();", Usage("IMeter.Read").Source);
        Assert.Contains("return Receiver(Value);", Usage("Transform.Invoke(System.Int32)").Source);
        Assert.Contains("Use(ref Receiver: Sample.Counter);", Usage("Counter.Advance").Source);
        var asyncCall = Usage("Counter.AdvanceAsync");
        Assert.Contains("Use(ref Receiver: Sample.Counter): Tasks.Task of int;", asyncCall.Source);
        Assert.Contains("return Receiver.AdvanceAsync();", asyncCall.Source);
        Assert.Contains("abstract", Usage("Base.#ctor").Reason);
        Assert.Contains("accessors", Usage("Widget.get_Size").Reason);
    }

    [Theory]
    [InlineData("--usage", "M:")]
    [InlineData("--name", "System.String.Length", "--access", "read")]
    [InlineData("--usage", "M:System.Object.ToString", "--access", "write")]
    [InlineData("--usage", "E:Example.Changed")]
    [InlineData("--name", "System.Array.Empty", "--type-arg", "int")]
    [InlineData("--usage", "M:X.Y", "--name", "X.Y")]
    [InlineData("--usage", "M:X.Y", "--match", "prefix")]
    public async Task Invalid_selection_does_not_prepare_a_project(params string[] options)
    {
        using var directory = new Fpas.Tests.Support.TemporaryDirectory();
        var result = await DocumentationFixture.Cli(directory, ["Missing.fpasproj", "--report", "json", .. options]);
        Assert.Equal(2, result.ExitCode);
        using var report = JsonDocument.Parse(result.Output);
        Assert.Equal("invalid_argument", report.RootElement.GetProperty("error").GetProperty("code").GetString());
    }

    [Fact]
    public async Task Generic_usage_binds_supplied_owner_and_method_types_in_declaration_order()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Convert.cs"), """
            namespace Sample;
            public class Box<T> where T : class, new()
            {
                public Box(T value) { }
                public U Read<U>(T original, U replacement) where U : struct => replacement;
                public static T Create() => new T();
            }
            """);
        const string member = "M:Sample.Box`1.Read``1(`0,``0)";
        const string list = "System.Collections.Generic.List of array of string";
        var response = await DocumentationFixture.Cli(directory, directory.File("Game/Game.fpasproj"),
            "--usage", member, "--type-arg", list, "--type-arg", "int", "--report", "json");
        Assert.True(response.ExitCode == 0, response.Output + response.Error);
        using var report = JsonDocument.Parse(response.Output);
        var usage = Assert.Single(report.RootElement.GetProperty("result").GetProperty("entries").EnumerateArray()).GetProperty("usage");
        Assert.Equal("template", usage.GetProperty("status").GetString());
        var source = usage.GetProperty("source").GetString();
        Assert.Contains("Receiver: Sample.Box of Generic.List of array of string", source);
        Assert.Contains("return Receiver.Read of int(Original, Replacement);", source);

        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        ApiUsage Select(string id, params string[] arguments) => Assert.IsType<ApiUsage>(
            Assert.Single(query.Search(new(id, Assembly: "Shared", Usage: true, TypeArguments: arguments)).Entries).Usage);
        Assert.Contains("new Sample.Box of Text.StringBuilder(Value)", Select("M:Sample.Box`1.#ctor(`0)", "System.Text.StringBuilder").Source);
        Assert.Contains("(Sample.Box of Text.StringBuilder).Create()", Select("M:Sample.Box`1.Create", "System.Text.StringBuilder").Source);
        var nestedId = Assert.Single(query.Search(new("Lookup.Api.Nested.Combine", Assembly: "Shared")).Entries).DocumentationId!;
        var nested = Select(nestedId, "System.Text.StringBuilder", "int", "string");
        Assert.Contains("Receiver: (Lookup.Api of Text.StringBuilder).Nested of int", nested.Source);
        Assert.Contains("Receiver.Combine of string(", nested.Source);
        Assert.Contains("declaring type (1) and method (1)", Select(member, list).Reason);
        var invalidOwner = Select(member, "int", "int");
        Assert.Equal("unavailable", invalidOwner.Status);
        Assert.StartsWith("FP3018:", invalidOwner.Reason);
        Assert.Contains("'class'", invalidOwner.Reason);
        Assert.Contains("constraints", Select(member, list, "string").Reason);
        Assert.Contains("FP3001", Select(member, "Unknown.Library", "int").Reason);
        Assert.Contains("Unexpected content", Select(member, list + ";", "int").Reason);
    }
}
