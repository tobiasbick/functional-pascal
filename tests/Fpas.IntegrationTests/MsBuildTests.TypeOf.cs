namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Runtime_type_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("TypeInspection"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "type-inspection")))
            File.WriteAllText(project.File("TypeInspection/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "TypeInspection/TypeInspection.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "TypeInspection/bin/Debug/net10.0/TypeInspection.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "System.Int32", "True", "True", "2", "String", ""), run.Output);
    }

    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Typeof_returns_real_CLR_types_across_projects_without_running_initializers(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public static class Log
            {
                public static int Count;
                public static int Mark() => ++Count;
            }
            public sealed class Foreign
            {
                static Foreign() => Log.Mark();
                public Foreign() => Log.Mark();
            }
            """);
        File.WriteAllText(project.File("Game/Models.fpas"), """
            unit Demo.Models;
            uses Shared as S;
            const Initialized: int := S.Log.Mark();
            public type Amount = distinct decimal;
            public type UserName = distinct string;
            public type Percent = int range 0..100;
            public type Direction = enum North; South; end enum;
            public type Model<T> = record Value: T; end record;
            public type Tree<T> = union Empty; Leaf(Value: T); end union;
            public function Touch(): int; begin return Initialized; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Types;
            uses Demo.Models as M, Shared as S, System as Sys, System.Threading.Tasks as Tasks;
            public const IntegerType: Sys.Type := typeof(int);
            type Hidden = record Value: int := S.Log.Mark(); end record;
            public function Snapshot(): Sys.Type[];
            begin
              return [typeof(M.Amount), typeof(M.UserName), typeof(M.Percent), typeof(M.Direction),
                typeof(M.Model<int>), typeof(M.Tree<string>), typeof(M.Tree<string>[]), typeof(S.Foreign)];
            end function;
            public function Get<T>(): Sys.Type; begin return typeof(T); end function;
            public function Factory<T>(): function(): Sys.Type; begin return function(): Sys.Type => typeof(T); end function;
            public function HiddenType(): Sys.Type; begin return typeof(Hidden); end function;
            public function Touch(): int; begin return M.Touch(); end function;
            public async function AsyncGet<T>(Gate: Tasks.Task): Tasks.Task<Sys.Type>;
            begin await Gate; return typeof(T); end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared;
            Type[] expected = [typeof(Models.Amount), typeof(Models.UserName), typeof(Models.Percent), typeof(Models.Direction),
                typeof(Models.Model<int>), typeof(Models.Tree<string>), typeof(Models.Tree<string>[]), typeof(Foreign)];
            Type[] actual = Types.Snapshot();
            if (!actual.SequenceEqual(expected) || Types.IntegerType != typeof(int)) throw new Exception("Wrong CLR type identity");
            if (Types.Get<string?>() != typeof(string) || Types.Get<int?>() != typeof(int?)) throw new Exception("Wrong nullable identity");
            if (Types.Factory<int>()() != typeof(int) || Types.Factory<string>()() != typeof(string)) throw new Exception("Generic closure lost type argument");
            Type hidden = Types.HiddenType();
            if (!hidden.IsNestedPrivate || hidden.Name != "Hidden" || Log.Count != 0) throw new Exception("typeof ran a constructor or unit initializer");
            var gate = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
            var pending = Types.AsyncGet<List<string>>(gate.Task);
            if (pending.IsCompleted) throw new Exception("Generic type-token test did not suspend");
            gate.SetResult();
            if (await pending.WaitAsync(TimeSpan.FromSeconds(10)) != typeof(List<string>)) throw new Exception("Async type parameter was erased");
            if (Log.Count != 0 || Types.Touch() != 1 || Log.Count != 1) throw new Exception("Type initialization behavior changed");
            Console.WriteLine("Typeof CLR behavior verified");
            """);
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration, "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, $"Host/bin/{configuration}/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Typeof CLR behavior verified" + Environment.NewLine, run.Output);
    }
}
