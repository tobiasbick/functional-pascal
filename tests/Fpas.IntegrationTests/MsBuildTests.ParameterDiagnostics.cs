namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    private const string ParameterApiSource = """
        namespace Demo;
        public delegate int Adjust(ref int value, out int previous, in int delta);
        public interface IReadCounter { int Read(in Counter value); }
        public sealed class ParameterApi : IReadCounter
        {
            public static int Field;
            public int Property { get; set; }
            public ParameterApi(int initial = 5) => Property = initial;
            public int Read(in Counter value) => value.Value;
            public static int Change(ref int value, out int previous, in int delta)
            { previous = value; value += delta; return value; }
            public static int Sum(int seed = 2, params int[] values) => seed + values.Sum();
            public static string Choose(int value) => "exact";
            public static string Choose(params int[] values) => "params";
            public static string Empty(params object[] values) => "objects";
            public static string Empty(params string[] values) => "strings";
            public static string Prefer(int first, params int[] rest) => "fixed";
            public static string Prefer(params int[] values) => "expanded";
            public static string Mode(int value) => "value";
            public static string Mode(in int value) => "in";
            public static bool Token(System.Threading.CancellationToken token = default) => token.CanBeCanceled;
            public static bool Day(DayOfWeek day = DayOfWeek.Monday) => day == DayOfWeek.Monday;
            public static int Optional(int first = 0) => 1;
            public static int Optional(int first = 0, int second = 0) => 2;
            public static void Clear(ref string? text) => text = null;
        }
        """;

    [Theory]
    [InlineData("discard D.ParameterApi.Optional();", "FP3003")]
    [InlineData("var Text: string? := 'known'; if Text <> null then D.ParameterApi.Clear(ref Text); discard Text.Length; end if;", "FP3013")]
    [InlineData("const Api: D.ParameterApi := new D.ParameterApi(); var Before: int := 0; const Step: int := 1; discard D.ParameterApi.Change(ref Api.Property, out Before, in Step);", "FP3028")]
    [InlineData("const Callback: D.Adjust := function(out Value: int; out Before: int; in Step: int): int begin Before := 0; Value := Step; return Value; end function;", "FP3019")]
    public async Task Invalid_imported_argument_modes_and_optional_ambiguities_are_diagnosed(string body, string code)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/ParameterApi.cs"), ParameterApiSource);
        File.WriteAllText(project.File("Game/Bridge.fpas"), $$"""
            unit Demo.Bridge;
            uses Demo as D;
            public procedure Invalid();
            begin {{body}} end procedure;
            end unit;
            """);
        var result = await sdk.Run(project, "build", "Game/Game.fpasproj");
        Assert.NotEqual(0, result.ExitCode);
        Assert.Contains(code, result.Output + result.Error);
    }
}
