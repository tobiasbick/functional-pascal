using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Contracts_run_in_the_callee_after_cleanup_for_CSharp_and_delegate_calls(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/ContractResource.cs"), """
            namespace Demo;
            public sealed class ContractResource(int[] state, int mode) : IDisposable
            {
                public void Dispose()
                {
                    state[2]++;
                    if (mode == 1) throw new ArgumentException("cleanup");
                }
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System as Sys, Demo as D;
            public function Guard(Value: int): int;
            requires Value > 0;
            ensures (Returned: int): Returned = Value;
            begin return Value; end function;
            public function Post(): int;
            ensures (Returned: int): Returned = 2;
            begin
              try return 1; catch Failure: Sys.InvalidOperationException do return 99; end try;
            end function;
            function Once(State: array of int): int;
            begin State[1] := State[1] + 1; return 7; end function;
            public function Cleanup(State: array of int; Mode: int): int;
            requires State.Length = 3;
            ensures (Returned: int): Returned = State[0];
            ensures State[1] = 1 and State[2] = 1 and Mode = 0;
            begin
              try
                using const Resource: D.ContractResource := new D.ContractResource(State, Mode);
                if Mode = 2 then raise new Sys.ArgumentException('body'); end if;
                return Once(State);
              finally State[0] := 7; end try;
            end function;
            public type Counter = record
              public Value: int;
              public function Positive(): int;
              requires self.Value > 0;
              ensures (Returned: int): Returned = self.Value;
              begin return self.Value; end function;
            end record;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            static void Require(bool value) { if (!value) throw new Exception("Wrong contract behavior."); }
            static void Fails(Action call, string clause)
            {
                try { call(); throw new Exception("Contract did not execute."); }
                catch (InvalidOperationException error) { Require(error.Message.Contains(clause, StringComparison.Ordinal)); }
            }
            Require(Bridge.Guard(3) == 3);
            Func<int, int> callback = Bridge.Guard;
            Fails(() => Bridge.Guard(0), "requires Value > 0");
            Fails(() => callback(0), "requires Value > 0");
            Fails(() => Bridge.Post(), "ensures Returned = 2");
            var counter = new Bridge.Counter(4);
            Func<int> method = counter.Positive;
            Require(method() == 4);
            Fails(() => new Bridge.Counter(0).Positive(), "requires self.Value > 0");
            var state = new int[3];
            Require(Bridge.Cleanup(state, 0) == 7 && state.SequenceEqual(new[] { 7, 1, 1 }));
            foreach (var mode in new[] { 1, 2 })
            {
                state = new int[3];
                try { Bridge.Cleanup(state, mode); throw new Exception("Missing exception."); }
                catch (ArgumentException error) { Require(error.Message == (mode == 1 ? "cleanup" : "body")); }
                Require(state[0] == 7 && state[2] == 1);
            }
            Console.WriteLine("Contracts checked");
            """);
        await BuildAndRunContractProject(project, configuration, "Contracts checked");
    }

    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Parameter_metadata_and_managed_storage_roundtrip_through_CSharp(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/ParameterApi.cs"), ParameterApiSource);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo as D;
            public function Change(ref Value: int; out Previous: int; in Delta: int): int;
            requires Delta > 0;
            ensures Value = Previous + Delta;
            ensures (Returned: int): Returned = Value;
            begin Previous := Value; Value := Value + Delta; return Value; end function;
            public function Sum(Seed: int := 1 + 2; params Values: array of int): int;
            begin var Total: int := Seed; for Value: int in Values do Total := Total + Value; end for; return Total; end function;
            public function Run(): int;
            begin
              var Value: int := 1; var Previous: int := 0; const Delta: int := 2;
              discard D.ParameterApi.Change(ref Value, out Previous, in Delta);
              const Callback: D.Adjust := Change;
              discard Callback(ref Value, out Previous, in Delta);
              const Lambda: D.Adjust := function(ref Item: int; out Before: int; in Step: int): int
                ensures Item = Before + Step;
                begin Before := Item; Item := Item + Step; return Item; end function;
              discard Lambda(ref Value, out Previous, in Delta);
              discard D.ParameterApi.Change(ref D.ParameterApi.Field, out Previous, in Delta);
              const Api: D.ParameterApi := new D.ParameterApi();
              const Counter: D.Counter := new D.Counter();
              const Reader: D.IReadCounter := Api;
              if Reader.Read(in Counter) <> 0 then return -1; end if;
              if D.ParameterApi.Choose(1) <> 'exact' then return -2; end if;
              if D.ParameterApi.Choose() <> 'params' then return -3; end if;
              if D.ParameterApi.Empty() <> 'strings' then return -4; end if;
              if D.ParameterApi.Prefer(1) <> 'fixed' then return -5; end if;
              if D.ParameterApi.Mode(Value) <> 'value' then return -6; end if;
              if D.ParameterApi.Mode(in Value) <> 'in' then return -7; end if;
              if D.ParameterApi.Token() then return -8; end if;
              if not D.ParameterApi.Day() then return -9; end if;
              return Value + D.ParameterApi.Sum(values := [3, 4]) + Api.Property;
            end function;
            public procedure Touch(in Value: D.Counter);
            begin Value.Increment(); end procedure;
            public function Nullable(Value: double? := 1; Name: string? := 'ready'): double;
            begin if Name <> null and Value.HasValue then return Value.Value; else return 0; end if; end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Middle"));
        File.WriteAllText(project.File("Middle/Middle.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Middle/Consumer.fpas"), """
            unit Demo.ParameterConsumer;
            uses Demo.Bridge as Bridge;
            public function Run(): int;
            begin
              var Value: int := 4; var Previous: int := 0; const Delta: int := 2;
              discard Bridge.Change(ref Value, out Previous, in Delta);
              return Value + Bridge.Sum(Values := [4, 5]);
            end function;
            end unit;
            """);
        var host = project.File("Host/Host.csproj");
        File.WriteAllText(host, File.ReadAllText(host).Replace("../Game/Game.fpasproj", "../Middle/Middle.fpasproj", StringComparison.Ordinal));
        var solution = System.Xml.Linq.XDocument.Load(project.File("Game.slnx"));
        solution.Root!.Add(new System.Xml.Linq.XElement("Project", new System.Xml.Linq.XAttribute("Path", "Middle/Middle.fpasproj")));
        solution.Save(project.File("Game.slnx"));
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            static void Require(bool value) { if (!value) throw new Exception("Wrong parameter behavior."); }
            var value = 4; var delta = 2;
            Require(Bridge.Change(ref value, out var previous, in delta) == 6 && previous == 4);
            Adjust callback = Bridge.Change;
            Require(callback(ref value, out previous, in delta) == 8 && previous == 6);
            Require(Bridge.Sum() == 3 && Bridge.Sum(1, 2, 3) == 6 && Bridge.Sum(Values: new[] { 4, 5 }) == 12);
            Require(Bridge.Run() == 21 && ParameterApi.Field == 2);
            Require(ParameterConsumer.Run() == 18);
            Require(Bridge.Nullable() == 1.0);
            Require(ParameterApi.Empty() == "strings" && ParameterApi.Prefer(1) == "fixed");
            var counter = new Counter { Value = 4 };
            Bridge.Touch(in counter);
            Require(counter.Value == 4);
            var parameters = typeof(Bridge).GetMethod(nameof(Bridge.Change))!.GetParameters();
            Require(parameters[0].ParameterType.IsByRef && parameters[1].IsOut && parameters[2].IsIn);
            Require(parameters[2].GetCustomAttributesData().Any(attribute => attribute.AttributeType.FullName == "System.Runtime.CompilerServices.IsReadOnlyAttribute"));
            var sum = typeof(Bridge).GetMethod(nameof(Bridge.Sum))!.GetParameters();
            Require(sum[0].IsOptional && Equals(sum[0].DefaultValue, 3));
            Require(sum[1].IsDefined(typeof(ParamArrayAttribute), false));
            Console.WriteLine("Parameters checked");
            """);
        await BuildAndRunContractProject(project, configuration, "Parameters checked");
    }

    private async Task BuildAndRunContractProject(TemporaryDirectory project, string configuration, string expected)
    {
        var build = await sdk.Run(project, "build", "Game.slnx", "--configuration", configuration);
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var result = await sdk.Run(project, $"Host/bin/{configuration}/net10.0/Host.dll");
        Assert.True(result.ExitCode == 0, result.Output + result.Error);
        Assert.Equal(expected + Environment.NewLine, result.Output);
        Assert.Empty(result.Error);
    }
}
