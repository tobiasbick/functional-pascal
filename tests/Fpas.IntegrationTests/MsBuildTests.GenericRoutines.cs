using System.Xml.Linq;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Generic_pipeline_example_builds_and_runs_composed_functions()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Pipeline"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "generic-pipeline")))
            File.WriteAllText(project.File("Pipeline/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Pipeline/GenericPipeline.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Pipeline/bin/Debug/net10.0/GenericPipeline.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "Value: 2", "Value: 4", "Value: 6", "identity", ""), run.Output);
    }

    [Fact]
    public async Task Generic_source_routines_constraints_closures_and_record_methods_interoperate_with_CSharp()
    {
        using var project = sdk.CreateProject();
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $"""
            <Project Sdk="Fpas.NET.Sdk/{sdk.Version}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Bridge as B, System as Sys;
            public function Roundtrip<T>(Value: T): T; begin return B.Forward<T>(Value); end function;
            public function Compare<T>(Left: T; Right: T): int;
            where T: Sys.IComparable<T>;
            begin return B.Compare<T>(Left, Right); end function;
            public function NullableText<T>(Value: T?): string?;
            where T: struct;
            begin return B.Pair<T>(Value, null).Item2; end function;
            end unit;
            """);
        var hostProject = project.File("Host/Host.csproj"); var hostXml = XDocument.Load(hostProject);
        hostXml.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); hostXml.Save(hostProject);
        File.WriteAllText(project.File("Shared/GenericTypes.cs"), """
            namespace Demo;
            public static class GenericNullable { public static Tuple<T?, string?> Pair<T>(T? value, string? text) where T : struct => new(value, text); }
            public interface IMutable { void Bump(); int Read(); }
            public struct Mutable(int value) : IMutable { public void Bump() => value++; public readonly int Read() => value; }
            public sealed class ReferenceMutable(int value) : IMutable { public void Bump() => value++; public int Read() => value; }
            public sealed class DisposableProbe : IDisposable { public bool Disposed { get; private set; } public void Dispose() => Disposed = true; }
            public class GenericBase { public int Number; public virtual int Read() => Number; }
            public sealed class GenericDerived : GenericBase { public override int Read() => Number * 2; }
            """);
        File.WriteAllText(project.File("Game/GenericLibrary.fpas"), """
            unit Demo.GenericLibrary;
            public function Identity<T>(Value: T): T; begin return Value; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses System as Sys, Demo as D, Demo.GenericLibrary as G;
            public function Forward<T>(Value: T): T; begin return G.Identity<T>(Value); end function;
            public function Callback<T>(): function(T): T; begin return G.Identity<T>; end function;
            public function Hold<T>(Value: T): function(): T; begin return function(): T => G.Identity<T>(Value); end function;
            public function Compare<T>(Left: T; Right: T): int;
            where T: Sys.IComparable<T>;
            begin return Left.CompareTo(Right); end function;
            public function Create<T>(): T; where T: new(); begin return new T(); end function;
            public function Ref<T>(Value: T): T; where T: class; begin return Value; end function;
            public function IsMissing<T>(Value: T): bool; where T: class?; begin return Value = null; end function;
            public function Optional<T>(Value: T?): T?; where T: class; begin return Value; end function;
            public function Pair<T>(Value: T?; Text: string?): Sys.Tuple<T?, string?>;
            where T: struct;
            begin return new Sys.Tuple<T?, string?>(Value, Text); end function;
            public function Dispose<T>(Value: T): int;
            where T: Sys.IDisposable;
            begin using const Resource: T := Value; return 7; end function;
            public function EnumHash<T>(Value: T): int;
            where T: struct, Sys.Enum;
            begin return Value.GetHashCode(); end function;
            public function Catch<T>(Failure: Sys.Exception): bool;
            where T: Sys.Exception;
            begin try raise Failure; catch Error: T do return true; catch do return false; end try; end function;
            public function Mutate<T>(Input: T): int;
            where T: D.IMutable;
            begin var Value: T := Input; Value.Bump(); Value.Bump(); return Value.Read(); end function;
            public function Snapshot<T>(Input: T): function(): int;
            where T: D.IMutable;
            begin var Value: T := Input; const Read: function(): int := Value.Read; Value.Bump(); return Read; end function;
            public function Saved<T>(Input: T): function(): int;
            where T: D.IMutable;
            begin var Value: T := Input; return function(): int begin Value.Bump(); return Value.Read(); end function; end function;
            public function BaseMember<T>(Value: T): int;
            where T: D.GenericBase;
            begin Value.Number := Value.Number + 1; return Value.Read(); end function;
            public function Upcast<T, U>(Value: T): U;
            where T: U;
            where U: class;
            begin return Value; end function;
            public type Holder = record
              Value: int;
              private function Inner<T>(Item: T): T; begin return Item; end function;
              function Echo<T>(Item: T): T; begin return Inner<T>(Item); end function;
              function Capture<T>(Item: T): function(): T;
              begin return function(): T begin discard self.Value; return self.Inner<T>(Item); end function; end function;
            end record;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            if (Bridge.Forward(42) != 42 || Bridge.Forward<string?>(null) != null || Bridge.Callback<int>()(3) != 3) throw new Exception("Source calls");
            if (Consumer.Roundtrip(42) != 42 || Consumer.Roundtrip<string?>(null) != null || Consumer.Compare(1, 2) >= 0 || Consumer.NullableText<int>(null) != null) throw new Exception("FPAS assembly boundary");
            if (Bridge.Hold(42)() != 42 || Bridge.Hold<string?>(null)() != null) throw new Exception("Generic closure");
            if (Bridge.Compare(1, 2) >= 0 || Bridge.Compare("a", "b") >= 0) throw new Exception("Interface dispatch");
            var disposable = new DisposableProbe();
            if (Bridge.Dispose(disposable) != 7 || !disposable.Disposed || Bridge.EnumHash(DayOfWeek.Friday) != 5) throw new Exception("Cleanup/enum constraint");
            if (!Bridge.Catch<InvalidOperationException>(new InvalidOperationException()) || Bridge.Catch<InvalidOperationException>(new Exception())) throw new Exception("Generic catch");
            var created = Bridge.Create<GenericDerived>();
            if (Bridge.Ref(created) != created || !Bridge.IsMissing<GenericDerived?>(null) || Bridge.Optional<string>(null) != null) throw new Exception("Reference constraints");
            var pair = Bridge.Pair<int>(null, null);
            if (pair.Item1 != null || pair.Item2 != null) throw new Exception("Nullable generic value");
            var value = new Mutable(1); var reference = new ReferenceMutable(1);
            if (Bridge.Mutate(value) != 3 || value.Read() != 1 || Bridge.Mutate(reference) != 3 || reference.Read() != 3) throw new Exception("Constrained mutation");
            if (Bridge.Snapshot(new Mutable(1))() != 1 || Bridge.Snapshot(new ReferenceMutable(1))() != 2) throw new Exception("Method group boxing");
            var saved = Bridge.Saved(new Mutable(1));
            if (saved() != 2 || saved() != 3) throw new Exception("Mutable generic capture");
            if (Bridge.BaseMember(new GenericDerived { Number = 2 }) != 6 || Bridge.Upcast<GenericDerived, GenericBase>(created) != created) throw new Exception("Base/dependent constraints");
            var holder = new Bridge.Holder(1);
            if (holder.Echo("text") != "text" || holder.Capture(42)() != 42) throw new Exception("Record generics");
            var method = typeof(Bridge).GetMethod("Compare")!;
            if (!method.IsGenericMethodDefinition || method.GetGenericArguments().Single().GetGenericParameterConstraints().Single().GetGenericTypeDefinition() != typeof(IComparable<>)) throw new Exception("Generic metadata");
            var csFlags = typeof(GenericNullable).GetMethod("Pair")!.ReturnParameter.GetCustomAttributesData().Single(attribute => attribute.AttributeType.FullName == "System.Runtime.CompilerServices.NullableAttribute").ConstructorArguments[0].ToString();
            var fpFlags = typeof(Bridge).GetMethod("Pair")!.ReturnParameter.GetCustomAttributesData().Single(attribute => attribute.AttributeType.FullName == "System.Runtime.CompilerServices.NullableAttribute").ConstructorArguments[0].ToString();
            if (csFlags != fpFlags) throw new Exception("Nullable layout C#: " + csFlags + " FPAS: " + fpFlags);
            Console.WriteLine("CLR source generics verified");
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("CLR source generics verified" + Environment.NewLine, run.Output);
        File.WriteAllText(project.File("Host/Invalid.cs"), """
            public static class Invalid
            {
                public static string Read() => Demo.Bridge.Pair<int>(null, null).Item2;
            }
            """);
        var nullable = await sdk.Run(project, "build", "Host/Host.csproj", "--no-restore", "-p:TreatWarningsAsErrors=true");
        Assert.NotEqual(0, nullable.ExitCode); Assert.Contains("CS8603", nullable.Output);
    }
}
