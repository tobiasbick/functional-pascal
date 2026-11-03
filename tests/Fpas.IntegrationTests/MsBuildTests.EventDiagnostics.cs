using System.Text.Json;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Inaccessible_hidden_ambiguous_and_invalid_event_operations_fail_with_source_diagnostics()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public class Base
            {
                public event Action Changed { add { } remove { } }
                protected event Action Protected { add { } remove { } }
            }
            public sealed class Private : Base { private new event Action Changed { add { } remove { } } }
            public sealed class Hidden : Base { public new Action Changed = () => { }; }
            public sealed class Invalid
            {
                public Action Field = () => { };
                public Action Property => Field;
                public event Action Event { add { } remove { } }
                public static event Action Static { add { } remove { } }
                public event Action<string?> Nullable { add { } remove { } }
            }
            public interface ILeft { event Action Changed; }
            public interface IRight { event Action Changed; }
            public interface IBoth : ILeft, IRight { }
            public interface IVariant<out T> { event Action<T> Changed; }
            public interface IVariantBoth : IVariant<string>, IVariant<object> { }
            public struct Counter { public event Action Changed { add { } remove { } } }
            """);
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        (string Declarations, string Body, string Code)[] cases =
        [
            ("", "subscribe new S.Private().Changed with Handler;", "FP3034"),
            ("", "subscribe new S.Base().Protected with Handler;", "FP3034"),
            ("", "subscribe new S.Hidden().Changed with Handler;", "FP3034"),
            ("", "subscribe new S.Invalid().Field with Handler;", "FP3034"),
            ("", "subscribe new S.Invalid().Property with Handler;", "FP3034"),
            ("", "subscribe new S.Invalid().Static with Handler;", "FP3034"),
            ("", "subscribe S.Invalid.Event with Handler;", "FP3034"),
            ("", "unsubscribe new S.Private().Changed with Handler;", "FP3034"),
            ("procedure Wire(Value: S.IBoth; Handler: Sys.Action); begin subscribe Value.Changed with Handler; end procedure;", "", "FP3034"),
            ("procedure Wire(Value: S.IVariantBoth; Handler: Sys.Action<object>); begin subscribe Value.Changed with Handler; end procedure;", "", "FP3034"),
            ("", "subscribe new S.Invalid().Nullable with procedure(Value: string) begin end procedure;", "FP3003"),
            ("async function Wire(Value: Tasks.Task<Sys.Action>): Tasks.Task; begin var Counter: S.Counter := new S.Counter(); subscribe Counter.Changed with await Value; end function;", "", "FP3030")
        ];
        foreach (var (declarations, body, code) in cases)
        {
            var text = $$"""
                program InvalidEvent;
                uses Shared as S, System as Sys, System.Threading.Tasks as Tasks;
                {{declarations}}
                begin
                  const Handler: Sys.Action := procedure() begin end procedure;
                  {{body}}
                end;
                end program;
                """;
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Invalid.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"], text);
            Assert.NotEqual(0, result.ExitCode);
            using var document = JsonDocument.Parse(result.Output);
            var diagnostic = Assert.Single(document.RootElement.GetProperty("diagnostics").EnumerateArray());
            Assert.Equal(code, diagnostic.GetProperty("code").GetString());
        }
    }
}
