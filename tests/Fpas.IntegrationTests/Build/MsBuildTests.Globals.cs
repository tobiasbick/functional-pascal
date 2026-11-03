using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Shared_state_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("State"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "shared-state")))
            File.WriteAllText(project.File("State/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "State/SharedState.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "State/bin/Debug/net10.0/SharedState.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "3", "1000", "0", ""), run.Output);
    }

    [Fact]
    public async Task Globals_round_trip_across_CSharp_and_Fpas_with_CLR_initialization_and_field_metadata()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public static class Log
            {
                public static string Trace = "";
                public static int Mark(int value) { Trace += value; return value; }
                public static int Attempts;
                public static int Fail() { Attempts++; throw new InvalidOperationException("initialization failed"); }
                public static int Read(in int value) => value;
                public static void Increment(ref long value) => value++;
                public static void Set(out long value) => value = 12;
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.State;
            uses Shared.Log as Log, System.Threading.Tasks as Tasks;
            public const Maximum: int := Log.Mark(1);
            public var Count: long := 2L;
            public const Last: int := Log.Mark(2);
            public var Label: string? := null;
            public const Labels: string?[] := [null, 'text'];
            private var Hidden: int := 7;
            public type Snapshot = record
              Count: long := Count;
              function Read(): long; begin return Count; end function;
            end record;
            public type Outcome = union None; Some(Value: int); end union;
            public type Mode = enum Idle; Ready; end enum;
            public type Port = ushort range 1..65535;
            public var Choice: Outcome := Outcome.Some(42);
            public var Current: Mode := Mode.Ready;
            public const Endpoint: Port := new Port(80);
            public const ReadCount: function(): long := function(): long => Count;
            public procedure Touch(); begin end procedure;
            public function Secret(): int; begin return Hidden; end function;
            public function SnapshotNow(): Snapshot; begin return new Snapshot(); end function;
            public async function ReadAsync(): Tasks.Task<long>;
            begin await Tasks.Task.Delay(1); return Count; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Other.fpas"), """
            unit Demo.Other;
            uses Shared.Log as Log;
            public const Marker: int := Log.Mark(3);
            public var Count: int := 99;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Broken.fpas"), """
            unit Demo.Broken;
            uses Shared.Log as Log;
            public const Value: int := Log.Fail();
            public procedure Touch(); begin end procedure;
            end unit;
            """);
        File.WriteAllText(project.File("Game/CycleA.fpas"), """
            unit Demo.CycleA;
            uses Demo.CycleB as B;
            public var Value: int := B.Value + 1;
            public function Read(): int; begin return Value; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/CycleB.fpas"), """
            unit Demo.CycleB;
            uses Demo.CycleA as A;
            public var Value: int := A.Value + 10;
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
            uses Demo.State as S, Demo.Other as O, Shared.Log as Log;
            public const Initial: long := S.Count;
            public procedure Update();
            begin Log.Increment(ref S.Count); Log.Set(out S.Count); S.Count := S.Count + 1L; S.Label := 'updated'; end procedure;
            public function Read(): long; begin return S.ReadCount(); end function;
            public function Limit(): int; begin return Log.Read(in S.Maximum); end function;
            public function Other(): int; begin return O.Count; end function;
            public function Snapshot(): S.Snapshot; begin return S.SnapshotNow(); end function;
            end unit;
            """);
        var hostPath = project.File("Host/Host.csproj"); var host = XDocument.Load(hostPath);
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); host.Save(hostPath);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using Shared;
            using System.Reflection;
            if (Log.Trace != "") throw new Exception("Initialization was eager");
            Parallel.For(0, 32, _ => State.Touch());
            if (Log.Trace != "12" || State.Maximum != 1 || State.Last != 2 || State.Secret() != 7) throw new Exception("Initialization order/count");
            if (Consumer.Initial != 2 || Consumer.Limit() != 1) throw new Exception("Imported readonly fields");
            Consumer.Update();
            if (State.Count != 13 || Consumer.Read() != 13 || State.Label != "updated" || Consumer.Snapshot().Count != 13 || Consumer.Snapshot().Read() != 13) throw new Exception("Shared field storage/defaults");
            State.Count = 20; Log.Increment(ref State.Count);
            if (Consumer.Read() != 21 || await State.ReadAsync() != 21) throw new Exception("CLR writes/async fields");
            if (State.Current != State.Mode.Ready || State.Endpoint.Value != 80 || State.Choice is not State.Outcome.SomeCase { Value: 42 }) throw new Exception("Nested field ownership");
            if (Consumer.Other() != 99 || Other.Marker != 3 || Log.Trace != "123") throw new Exception("Independent unit ownership");
            for (int attempt = 0; attempt < 2; attempt++)
                try { Broken.Touch(); throw new Exception("Failed initializer executed body"); }
                catch (TypeInitializationException ex) when (ex.InnerException is InvalidOperationException { Message: "initialization failed" }) { }
            if (Log.Attempts != 1) throw new Exception("Failed initialization was retried");
            if (CycleA.Read() != 11 || CycleB.Value != 10) throw new Exception("CLR initialization cycles");
            var immutable = typeof(State).GetField("Maximum")!;
            if (!immutable.IsInitOnly || immutable.IsLiteral || !immutable.IsStatic || typeof(State).GetField("Count")!.IsInitOnly) throw new Exception("Field attributes");
            if (typeof(State).GetField("Hidden") != null || typeof(State).TypeInitializer == null || typeof(State).Attributes.HasFlag(TypeAttributes.BeforeFieldInit)) throw new Exception("Visibility/initialization metadata");
            Console.WriteLine("Global interop verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Global interop verified" + Environment.NewLine, run.Output);

        File.WriteAllText(project.File("Host/Invalid.cs"), "public static class Invalid { public static void Write() { Demo.State.Maximum = 9; } public static string Read() => Demo.State.Labels[0]; }");
        var invalid = await sdk.Run(project, "build", "Host/Host.csproj", "--no-restore", "-p:TreatWarningsAsErrors=true");
        Assert.NotEqual(0, invalid.ExitCode); Assert.Contains("CS0198", invalid.Output); Assert.Contains("CS8603", invalid.Output);
    }
}
