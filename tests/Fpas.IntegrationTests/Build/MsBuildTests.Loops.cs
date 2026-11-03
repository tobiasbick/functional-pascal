namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Typed_loops_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Loops"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "loops")))
            File.WriteAllText(project.File("Loops/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Loops/Loops.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Loops/bin/Debug/net10.0/Loops.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "1", "2", "3", "repeat: 3", "18446744073709551614", "18446744073709551615", ""), run.Output);
    }

    [Fact]
    public async Task Loop_exits_complete_cleanup_before_conditions_and_steps_in_sync_and_async_code()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public static class Log
            {
                public static string Trace = "";
                public static bool Check(int count) { Trace += "?"; return count == 3; }
                public static async Task<bool> CheckAsync(int count) { await Task.Yield(); return Check(count); }
                public static bool Fail() { Trace += "?"; throw new InvalidOperationException("condition"); }
            }
            public sealed class Resource(string name, bool fail = false) : IDisposable, IAsyncDisposable
            {
                private readonly int opened = Open(name);
                private static int Open(string name) { Log.Trace += name + "+"; return 1; }
                public void Dispose() { _ = opened; Log.Trace += name + "-"; if (fail) throw new InvalidOperationException("cleanup"); }
                public async ValueTask DisposeAsync() { await Task.Yield(); Dispose(); }
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Loops;
            uses Shared as S, System.Threading.Tasks as Tasks;
            type High = ulong range 18446744073709551614UL..18446744073709551615UL;
            public function RepeatSync(): int;
            begin
              var Count: int := 0;
              repeat
                using const Resource: S.Resource := new S.Resource('r');
                Count := Count + 1;
                if Count < 3 then continue; end if;
              until S.Log.Check(Count);
              return Count;
            end function;
            public async function RepeatAsync(): Tasks.Task<int>;
            begin
              var Count: int := 0;
              repeat
                await using const Resource: S.Resource := new S.Resource('a');
                Count := Count + 1;
                if Count < 3 then continue; end if;
              until await S.Log.CheckAsync(Count);
              return Count;
            end function;
            public procedure BreakEarly();
            begin repeat using const Resource: S.Resource := new S.Resource('b'); break; until S.Log.Fail(); end procedure;
            public function ReturnEarly(): int;
            begin repeat using const Resource: S.Resource := new S.Resource('e'); return 7; until S.Log.Fail(); end function;
            public procedure FailedCondition();
            begin repeat using const Resource: S.Resource := new S.Resource('c'); until S.Log.Fail(); end procedure;
            public async function FailedCleanup(): Tasks.Task;
            begin repeat await using const Resource: S.Resource := new S.Resource('f', true); continue; until S.Log.Fail(); end function;
            public procedure CountSync();
            begin
              for Index: byte := 254 to 255 do
                using const Resource: S.Resource := new S.Resource('s'); continue;
              end for;
            end procedure;
            public async function CountAsync(): Tasks.Task;
            begin
              for Index: High := new High(18446744073709551614UL) to new High(18446744073709551615UL) do
                await using const Resource: S.Resource := new S.Resource('h'); continue;
              end for;
            end function;
            public function FinallyOrder(): int;
            begin
              var Count: int := 0;
              repeat
                try Count := Count + 1; continue;
                finally S.Log.Trace := S.Log.Trace + '!'; end try;
              until S.Log.Check(Count);
              return Count;
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared;
            if (Loops.RepeatSync() != 3 || Log.Trace != "r+r-?r+r-?r+r-?") throw new Exception("Sync repeat ordering: " + Log.Trace);
            Log.Trace = "";
            if (await Loops.RepeatAsync() != 3 || Log.Trace != "a+a-?a+a-?a+a-?") throw new Exception("Async repeat ordering: " + Log.Trace);
            Log.Trace = ""; Loops.BreakEarly(); if (Log.Trace != "b+b-") throw new Exception("Break checked condition");
            Log.Trace = ""; if (Loops.ReturnEarly() != 7 || Log.Trace != "e+e-") throw new Exception("Return cleanup");
            Log.Trace = "";
            try { Loops.FailedCondition(); throw new Exception("Missing failure"); }
            catch (InvalidOperationException ex) when (ex.Message == "condition") { }
            if (Log.Trace != "c+c-?") throw new Exception("Condition failed before disposal");
            Log.Trace = "";
            try { await Loops.FailedCleanup(); throw new Exception("Missing failure"); }
            catch (InvalidOperationException ex) when (ex.Message == "cleanup") { }
            if (Log.Trace != "f+f-") throw new Exception("Condition ran after cleanup failure");
            Log.Trace = ""; Loops.CountSync(); if (Log.Trace != "s+s-s+s-") throw new Exception("Byte steps/cleanup");
            Log.Trace = ""; await Loops.CountAsync(); if (Log.Trace != "h+h-h+h-") throw new Exception("Subrange steps/async cleanup");
            Log.Trace = ""; if (Loops.FinallyOrder() != 3 || Log.Trace != "!?!?!?") throw new Exception("Finally/continue ordering");
            Console.WriteLine("Loop CLR behavior verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Loop CLR behavior verified" + Environment.NewLine, run.Output);
    }
}
