// [Documentation](../../../docs/pascal/tools/api-documentation.md)
using Fpas.Compiler.Analysis;
using Fpas.IntegrationTests.Build;
using Fpas.Workspaces.Documentation;

namespace Fpas.IntegrationTests.ApiDocumentation;

public sealed class NamedArgumentUsageTests(SdkPackageFixture sdk) : IClassFixture<SdkPackageFixture>
{
    [Fact]
    public async Task Shared_input_types_use_exact_labels_for_methods_constructors_and_delegates()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Calls.cs"), """
            #nullable enable
            namespace Sample;
            public delegate int Compare(int left, int right);
            public class Calls
            {
                public Calls(bool visible, bool enabled) { }
                public void Range(string receiver, int length, string? end) { }
                public static void Copy(ref int first, out int next, in int last) { next = last; }
                public static void Merge(int[] items, params int[] rest) { }
                public static void Pair<T, U>(T first, U second) { }
            }
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        ApiUsage Select(string member, params string[] types) => Assert.IsType<ApiUsage>(
            Assert.Single(query.Search(new("M:Sample." + member, Assembly: "Shared", Usage: true, TypeArguments: types)).Entries).Usage);

        Assert.Contains("new Sample.Calls(visible := Visible, enabled := Enabled)",
            Select("Calls.#ctor(System.Boolean,System.Boolean)").Source);
        var range = Select("Calls.Range(System.String,System.Int32,System.String)");
        Assert.Equal(["Receiver", "Receiver2", "Length", "End"], range.Inputs);
        Assert.Contains("Receiver.Range(receiver := Receiver2, length := Length, @end := End);", range.Source);
        Assert.Contains("Receiver(left := Left, right := Right)", Select("Compare.Invoke(System.Int32,System.Int32)").Source);
        Assert.Contains("Copy(first := ref First, next := out Next, last := in Last)",
            Select("Calls.Copy(System.Int32@,System.Int32@,System.Int32@)").Source);
        Assert.Contains("Merge(items := Items, rest := Rest)", Select("Calls.Merge(System.Int32[],System.Int32[])").Source);
        var substring = Assert.Single(query.Search(new("M:System.String.Substring(System.Int32,System.Int32)", Usage: true)).Entries);
        Assert.Contains("Receiver.Substring(startIndex := StartIndex, length := Length)", substring.Usage!.Source);
        const string pair = "Calls.Pair``2(``0,``1)";
        Assert.Contains("Pair of (int, int)(first := First, second := Second)", Select(pair, "int", "int").Source);
        Assert.Contains("Pair of (int, string)(First, Second)", Select(pair, "int", "string").Source);
    }

    [Fact]
    public async Task Keeps_a_checked_positional_call_when_labels_are_unusable_ambiguous_or_select_another_overload()
    {
        using var directory = DocumentationFixture.Create(sdk);
        File.WriteAllText(directory.File("Shared/Calls.cs"), """
            namespace Sample;
            public static class Calls
            {
                public static void Skip(int _, int count) { }
                public static void Draw(int x, int y, string color) { }
                public static void Draw(string color, int x, int y) { }
                public static void Send(System.ReadOnlySpan<byte> block, byte[] tail, int count) { }
                public static void Send(int count, byte[] tail, byte[] block) { }
            }
            """);
        var query = new DocumentationQuery(await DocumentationFixture.Analyze(directory));
        string? Source(string member) => Assert.IsType<ApiUsage>(
            Assert.Single(query.Search(new("M:Sample.Calls." + member, Assembly: "Shared", Usage: true)).Entries).Usage).Source;

        Assert.Contains("Skip(Argument1, Count)", Source("Skip(System.Int32,System.Int32)"));
        Assert.Contains("Draw(X, Y, Color)", Source("Draw(System.Int32,System.Int32,System.String)"));
        Assert.Contains("Draw(Color, X, Y)", Source("Draw(System.String,System.Int32,System.Int32)"));
        Assert.Contains("Send(Block, Tail, Count)", Source("Send(System.ReadOnlySpan{System.Byte},System.Byte[],System.Int32)"));
        Assert.Contains("Send(count := Count, tail := Tail, block := Block)", Source("Send(System.Int32,System.Byte[],System.Byte[])"));
    }
}
