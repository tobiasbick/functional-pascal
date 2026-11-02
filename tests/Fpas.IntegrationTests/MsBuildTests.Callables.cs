using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Delegates_custom_delegate_targets_generic_methods_and_closures_cross_the_CLR_boundary()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System as Sys, Demo as D, Demo.Helpers as H, System.Console as Console;
            public function Apply(Transform: function(int): int; Value: int): int;
            begin return Transform(Value); end function;
            public function Counter(Start: int): function(): int;
            begin var Value: int := Start; return function(): int begin Value := Value + 1; return Value; end function; end function;
            public function Prefix(Prefix: string): Sys.Func<string?, string>;
            begin return function(Value: string?): string => Prefix + (Value ?? 'null'); end function;
            public function StructCapture(): int;
            begin
              var Counter: D.Counter := new D.Counter();
              const Increment: procedure() := procedure() begin Counter.Increment(); end procedure;
              Counter.Value := 2; Increment(); Counter.Increment();
              const Snapshot: D.Counter := Counter;
              const Boxed: procedure() := Snapshot.Increment;
              Boxed();
              return Counter.Value + Snapshot.Value;
            end function;
            public function Other(): int;
            begin return H.Make()(3); end function;
            public function Run(): string;
            begin
              const Holder: D.DelegateHolder := new D.DelegateHolder(function(Value: int): int => Value + 1);
              const Owner: D.MethodOwner<int> := new D.MethodOwner<int>(6);
              const Abs: function(int): int := Sys.Math.Abs;
              const Transform: function(int): int := D.CallableApi.Receiver().Transform;
              const Convert: Sys.Func<string, string> := D.CallableApi.MakePrefix('CLR:');
              const Echo: function(int): int := Owner.Echo<int>;
              const Generic: int := D.CallableApi.Apply<int, int>(3, function(Value: int): int => Value * 2);
              const Formatted: string := D.CallableApi.Format(function(Value: int): string => Value.ToString());
              const Selected: string := D.CallableApi.Overload(function(Value: int): string => Value.ToString());
              const Callback: string := D.CallableApi.Run(Console.WriteLine);
              const Text: string := Owner.Convert<string>(function(Value: int): string => Value.ToString());
              return Formatted + ':' + Selected + ':' + Text + ':' + Convert('value') + ':' +
                (Abs(-2) + Transform(3) + Holder.Transform(4) + Generic + Echo(1) + D.CallableApi.ReadTransform(1)).ToString();
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Helpers.fpas"), """
            unit Demo.Helpers;
            private function Adjust(Value: int): int;
            begin return Value + 5; end function;
            public function Make(): function(int): int;
            begin return function(Value: int): int => Adjust(Value); end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            if (Bridge.Apply(value => value + 5, 2) != 7) throw new Exception("C# callback");
            if (Bridge.Other() != 8) throw new Exception("Multiple source closure ownership");
            if (Bridge.StructCapture() != 8) throw new Exception("Struct capture storage/copy");
            Func<int> first = Bridge.Counter(10), second = Bridge.Counter(20);
            if (first() != 11 || first() != 12 || second() != 21) throw new Exception("Closure lifetime");
            Func<string?, string> prefix = Bridge.Prefix("FPAS:");
            if (prefix(null) != "FPAS:null" || prefix("text") != "FPAS:text") throw new Exception("Nullable callback");
            if (Bridge.Run() != "7:formatter:8:6:CLR:value:34") throw new Exception("CLR delegate or method binding");
            if (CallableApi.ReceiverLog != "receiver;getter;") throw new Exception(CallableApi.ReceiverLog);
            Console.WriteLine("CLR callables verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("callback" + Environment.NewLine + "CLR callables verified" + Environment.NewLine, run.Output);
    }

    [Fact]
    public async Task Generic_method_constraints_and_callback_overloads_reject_unsafe_calls()
    {
        using var project = sdk.CreateProject();
        var build = await sdk.Run(project, "build", "Shared/Shared.csproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var cases = new (string Body, string? Error)[]
        {
            ("discard D.CallableApi.Dispose<string>('text');", "FP3018"),
            ("discard D.CallableApi.Create<string>();", "FP3018"),
            ("discard D.CallableApi.Require<string?>(null);", "FP3018"),
            ("discard D.CallableApi.Ambiguous(function(Value: int): bool => true);", "FP3003"),
            ("const F: function(int): bool := function(Value: int): bool => true; discard D.CallableApi.Format(F);", "FP3003"),
            ("discard D.CallableApi.Identity<string?>(null);", null),
            ("discard D.CallableApi.Create<int>();", null),
            ("discard D.CallableApi.Apply<int, string>(1, function(Value: int): string => Value.ToString());", null),
            ("discard D.CallableApi.NullCallbacks(function(Value: string): string => Value);", "FP3003"),
            ("discard D.CallableApi.NullCallbacks(function(Value: string?): string? => Value);", null)
        };
        foreach (var (body, error) in cases)
        {
            var result = await ProcessRunner.RunAsync([TestEnvironment.Metadata("FpasCliPath"), "compile", "-", "--output", project.File("Check.dll"),
                "--reference-dir", TestEnvironment.ReferenceDirectory, "--reference", project.File("Shared/bin/Debug/net10.0/Shared.dll"), "--diagnostics", "json"],
                "program Check; uses Demo as D; begin " + body + " end; end program;");
            if (error is null) Assert.True(result.ExitCode == 0, body + ": " + result.Output + result.Error);
            else
            {
                Assert.NotEqual(0, result.ExitCode);
                Assert.Contains("\"code\":\"" + error + "\"", result.Output + result.Error);
            }
        }
    }
}
