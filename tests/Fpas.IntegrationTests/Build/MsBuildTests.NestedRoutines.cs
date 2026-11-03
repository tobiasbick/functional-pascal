using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Local_functions_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Local"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "local-functions")))
            File.WriteAllText(project.File("Local/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Local/LocalFunctions.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Local/bin/Debug/net10.0/LocalFunctions.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "120", "11", "12", ""), run.Output);
    }

    [Fact]
    public async Task Nested_routines_escape_to_CSharp_and_separate_Fpas_consumers_with_their_CLR_contracts()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public delegate void RefWriter(ref int target, int amount);
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Factory;
            uses System as Sys, Shared as S, System.Threading.Tasks as Tasks;
            public function Counter(Start: int): Sys.Func<int>;
            begin
              var Value: int := Start;
              function Read(): int; begin return Next(); end function;
              function Next(): int; begin Value := Value + 1; return Value; end function;
              return Read;
            end function;
            public function Writer(Base: int): S.RefWriter;
            begin
              procedure Add(ref Target: int; Amount: int); begin Target := Target + Amount + Base; end procedure;
              return Add;
            end function;
            public function Positive(Limit: int): Sys.Func<int, int>;
            begin
              function Check(Value: int): int;
              requires Value >= 0;
              requires Value <= Limit;
              ensures (Returned: int): Returned = Value + 1;
              begin return Value + 1; end function;
              return Check;
            end function;
            public function ReadLater(Value: Tasks.Task<int>): Tasks.Task<int>;
            begin
              async function Run(): Tasks.Task<int>; begin return (await Value) + 1; end function;
              return Run();
            end function;
            public type Box<T> = record
              Value: T;
              function Capture<U>(Other: U): Sys.Func<T>;
              begin
                function Pick<V>(Extra: V): T; where V: struct;
                begin discard Other; discard Extra; return self.Value; end function;
                function Read(): T; begin return Pick<int>(2); end function;
                return Read;
              end function;
            end record;
            public function Format(): Sys.Func<int, string>;
            begin
              function Convert(Value: int): string; begin return Value.ToString(); end function;
              function Convert(Value: string): string; begin return Value; end function;
              return Convert;
            end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Factory as F;
            public function Read(): string;
            begin
              const Box: F.Box<string> := new F.Box<string>('saved');
              return Box.Capture<int>(3)() + F.Format()(42);
            end function;
            end unit;
            """);
        var hostPath = project.File("Host/Host.csproj"); var host = XDocument.Load(hostPath);
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); host.Save(hostPath);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            var first = Factory.Counter(10); var second = Factory.Counter(0);
            if (first() != 11 || first() != 12 || second() != 1) throw new Exception("Escaping cells");
            int value = 0; Factory.Writer(3)(ref value, 2);
            if (value != 5) throw new Exception("Byref nested delegate");
            var check = Factory.Positive(3);
            if (check(2) != 3) throw new Exception("Nested contract result");
            try { _ = check(4); throw new Exception("Missing contract failure"); }
            catch (InvalidOperationException ex) when (ex.Message.Contains("Check") && ex.Message.Contains("requires")) { }
            var gate = new TaskCompletionSource<int>(); var pending = Factory.ReadLater(gate.Task);
            if (pending.IsCompleted) throw new Exception("Expected suspension");
            gate.SetResult(41); if (await pending != 42) throw new Exception("Async receiver");
            var box = new Factory.Box<string>("typed");
            if (box.Capture(42)() != "typed" || Consumer.Read() != "saved42") throw new Exception("Type/method generic slots");
            if (typeof(Factory).GetMethods().Any(m => m.Name is "Read" or "Next" or "Check" or "Run" or "Convert")) throw new Exception("Nested API leaked");
            Console.WriteLine("Nested CLR interop verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Nested CLR interop verified" + Environment.NewLine, run.Output);
    }
}
