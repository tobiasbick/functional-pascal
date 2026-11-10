using System.Xml.Linq;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Attribute_example_builds_and_changes_real_JSON_output()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Attributes"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "attributes")))
            File.WriteAllText(project.File("Attributes/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Attributes/Attributes.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Attributes/bin/Debug/net10.0/Attributes.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("{\"sample_name\":\"Alpha\",\"Value\":42}" + Environment.NewLine, run.Output);
    }

    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Source_attributes_have_real_CLR_values_targets_and_runtime_behavior(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public enum Mode : ulong { High = ulong.MaxValue }
            [AttributeUsage(AttributeTargets.All, AllowMultiple = true)]
            public sealed class PayloadAttribute(string? text, Type? type, Mode mode, object? boxed, int[]? numbers, object?[]? values) : Attribute
            {
                public string? Text { get; } = text;
                public Type? Type { get; } = type;
                public Mode Mode { get; } = mode;
                public object? Boxed { get; } = boxed;
                public int[]? Numbers { get; } = numbers;
                public object?[]? Values { get; } = values;
                public string? Note { get; set; }
                public Type? ExtraType;
                public Mode[]? Modes { get; set; }
                public uint Count;
            }
            [AttributeUsage(AttributeTargets.All)]
            public sealed class SingleAttribute(int value) : Attribute { public int Value { get; } = value; }
            [AttributeUsage(AttributeTargets.All)]
            public sealed class GenericAttribute<T>(T value) : Attribute { public T Value { get; } = value; }
            [AttributeUsage(AttributeTargets.All)]
            public sealed class OptionalAttribute(int value = 7) : Attribute { public int Value { get; } = value; }
            [AttributeUsage(AttributeTargets.All, AllowMultiple = true)]
            public sealed class PackAttribute(params int[] values) : Attribute { public int[] Values { get; } = values; }
            [AttributeUsage(AttributeTargets.All)]
            public sealed class OverloadAttribute : Attribute
            {
                public string Kind { get; }
                public OverloadAttribute(int value) { Kind = "int:" + value; }
                public OverloadAttribute(long value) { Kind = "long:" + value; }
            }
            public sealed class ExplodingAttribute : Attribute
            {
                public ExplodingAttribute() => throw new InvalidOperationException("Attribute construction is a runtime operation");
            }
            public sealed class ScalarsAttribute(sbyte a, byte b, short c, ushort d, int e, uint f, long g, ulong h, float i, double j, char k, bool l) : Attribute
            {
                public object[] Values { get; } = [a, b, c, d, e, f, g, h, i, j, k, l];
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Marked;
            uses Shared as S, System as Sys, System.Threading.Tasks as Tasks,
              System.Collections.Generic as C, System.Text.Json.Serialization as Json, System.Runtime.CompilerServices as Compiler;
            [Sys.FlagsAttribute]
            public type Local = enum : uint A = 1; B = 2; end enum;
            [S.GenericAttribute<int>(42)]
            [S.OptionalAttribute]
            [S.PackAttribute(1, 2, 3)]
            [S.PackAttribute([])]
            [S.OverloadAttribute(5)]
            [Sys.SerializableAttribute]
            [S.ScalarsAttribute(-128, 255, -32768, 65535, -2147483648, 4294967295U, -9223372036854775808L,
              18446744073709551615UL, 1.25F, 2.5, #65535, true)]
            [S.PayloadAttribute('hello' + #10 + 'CLR', typeof(C.List<string?>), S.Mode.High, typeof(int?), [1, 2],
              [null, 'text', 42, true, #65, S.Mode.High, typeof(Model<int>), [1, 2], typeof(array of Model<int>), typeof(array[,] of int)],
              Note := 'named', ExtraType := typeof(Local), Modes := [S.Mode.High], Count := 4294967295U)]
            [S.PayloadAttribute(null, null, S.Mode.High, Local.A, null, null)]
            public type Model<T> = record
              [Json.JsonPropertyNameAttribute('payload')]
              public Value: T;
              [S.SingleAttribute(9)]
              [Compiler.MethodImplAttribute(Compiler.MethodImplOptions.AggressiveInlining)]
              public function Read(): T; begin return self.Value; end function;
            end record;
            [Sys.SerializableAttribute] [S.SingleAttribute(1)] public type Outcome<T> = variant Empty; Value(Data: T); end variant;
            [S.SingleAttribute(2)] public type Id = distinct long;
            [S.SingleAttribute(3)] public type Percent = int range 0..100;
            [S.SingleAttribute(4)] public type Name = distinct string;
            [S.SingleAttribute(5)]
            [Compiler.MethodImplAttribute(Compiler.MethodImplOptions.NoInlining)]
            public async function Work(): Tasks.Task<int>; begin await Tasks.Task.Delay(1); return 5; end function;
            [S.ExplodingAttribute]
            public function Explosive(): int; begin return 42; end function;
            public function Nested(): int;
              [S.SingleAttribute(6)]
              [Compiler.MethodImplAttribute(Compiler.MethodImplOptions.NoOptimization)]
              function Local(): int; begin return 6; end function;
            begin
              [procedure() begin end procedure][0]();
              return Local();
            end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $$"""
            <Project Sdk="Fpas.NET.Sdk/{{sdk.Version}}">
              <PropertyGroup><TargetFramework>net10.0</TargetFramework><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Marked as M;
            public function Read(): int;
            begin const Value: M.Model<int> := new M.Model<int>(42); return Value.Read(); end function;
            end unit;
            """);
        var host = XDocument.Load(project.File("Host/Host.csproj"));
        host.Root!.Element("ItemGroup")!.Add(new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")));
        host.Save(project.File("Host/Host.csproj"));
        var solution = XDocument.Load(project.File("Game.slnx"));
        solution.Root!.Add(new XElement("Project", new XAttribute("Path", "Consumer/Consumer.fpasproj"))); solution.Save(project.File("Game.slnx"));
        File.WriteAllText(project.File("Host/Program.cs"), """
            using System.Reflection;
            using System.Runtime.CompilerServices;
            using System.Text.Json;
            using System.Text.Json.Serialization;
            using Demo;
            using Shared;
            var type = typeof(Marked.Model<int>);
            if (Consumer.Read() != 42) throw new Exception("Annotated FPAS record could not be imported by a second FPAS project");
            var all = type.GetCustomAttributes<PayloadAttribute>().ToArray();
            var value = all[0];
            if (value.Text != "hello\nCLR" || value.Type != typeof(List<string>) || value.Mode != Mode.High ||
                value.Boxed is not Type boxed || boxed != typeof(int?) || !value.Numbers!.SequenceEqual([1, 2]) ||
                value.Note != "named" || value.ExtraType != typeof(Marked.Local) || value.Count != uint.MaxValue || value.Modes![0] != Mode.High)
                throw new Exception("Wrong fixed/named attribute values");
            var values = value.Values!;
            if (values[0] != null || (string)values[1]! != "text" || (int)values[2]! != 42 || (bool)values[3]! != true ||
                (char)values[4]! != 'A' || (Mode)values[5]! != Mode.High || (Type)values[6]! != type ||
                values[7] is not int[] array || !array.SequenceEqual([1, 2]) || (Type)values[8]! != type.MakeArrayType() ||
                (Type)values[9]! != typeof(int[,])) throw new Exception("Wrong boxed attribute payload");
            if (all[1].Text != null || all[1].Type != null || !Equals(all[1].Boxed, Marked.Local.A) || all[1].Numbers != null || all[1].Values != null)
                throw new Exception("Wrong null/boxed source enum attribute values");
            if (type.GetCustomAttribute<GenericAttribute<int>>()!.Value != 42 || type.GetCustomAttribute<OptionalAttribute>()!.Value != 7 ||
                type.GetCustomAttribute<OverloadAttribute>()!.Kind != "int:5") throw new Exception("Wrong generic/optional/overloaded constructor");
            var packs = type.GetCustomAttributes<PackAttribute>().ToArray();
            object[] expectedScalars = [sbyte.MinValue, byte.MaxValue, short.MinValue, ushort.MaxValue, int.MinValue, uint.MaxValue,
                long.MinValue, ulong.MaxValue, 1.25f, 2.5d, char.MaxValue, true];
            if (!type.GetCustomAttribute<ScalarsAttribute>()!.Values.SequenceEqual(expectedScalars)) throw new Exception("Wrong scalar widths or signedness");
            if (packs.Length != 2 || !packs[0].Values.SequenceEqual([1, 2, 3]) || packs[1].Values.Length != 0) throw new Exception("Wrong params array encoding");
            var property = type.GetProperty("Value")!;
            if (property.GetCustomAttribute<JsonPropertyNameAttribute>()?.Name != "payload" ||
                property.GetMethod!.GetCustomAttribute<JsonPropertyNameAttribute>() != null ||
                type.GetField("<Value>k__BackingField", BindingFlags.Instance | BindingFlags.NonPublic)!.GetCustomAttribute<JsonPropertyNameAttribute>() != null)
                throw new Exception("Record field annotation landed on the wrong CLR member");
            if (JsonSerializer.Serialize(new Marked.Model<int>(42)) != "{\"payload\":42}") throw new Exception("JSON ignored FPAS attribute");
            if (!type.IsDefined(typeof(SerializableAttribute)) ||
                !Marked.Outcome<int>.Value(1).GetType().IsDefined(typeof(SerializableAttribute)) ||
                !type.GetMethod("Read")!.MethodImplementationFlags.HasFlag(MethodImplAttributes.AggressiveInlining))
                throw new Exception("Pseudo-attributes were not mapped to CLR flags");
            if (type.GetMethod("Read")!.GetCustomAttribute<SingleAttribute>()?.Value != 9 ||
                typeof(Marked.Outcome<int>).GetCustomAttribute<SingleAttribute>()?.Value != 1 ||
                typeof(Marked.Id).GetCustomAttribute<SingleAttribute>()?.Value != 2 ||
                typeof(Marked.Percent).GetCustomAttribute<SingleAttribute>()?.Value != 3 ||
                typeof(Marked.Name).GetCustomAttribute<SingleAttribute>()?.Value != 4 ||
                !typeof(Marked.Local).IsDefined(typeof(FlagsAttribute))) throw new Exception("Wrong type/method attribute target");
            var work = typeof(Marked).GetMethod("Work")!;
            if (work.GetCustomAttribute<SingleAttribute>()?.Value != 5 || !work.MethodImplementationFlags.HasFlag(MethodImplAttributes.NoInlining) || await Marked.Work() != 5 ||
                work.GetCustomAttribute<AsyncStateMachineAttribute>()!.StateMachineType.GetMethod("MoveNext")!.IsDefined(typeof(SingleAttribute)))
                throw new Exception("Async annotation moved to the generated body");
            if (Marked.Explosive() != 42 || !typeof(Marked).GetMethod("Explosive")!.CustomAttributes.Any(item => item.AttributeType == typeof(ExplodingAttribute)))
                throw new Exception("Compiler executed an attribute constructor");
            if (Marked.Nested() != 6 || !typeof(Marked).GetNestedTypes(BindingFlags.NonPublic).SelectMany(item =>
                item.GetMethods(BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic)).Any(item => item.GetCustomAttribute<SingleAttribute>()?.Value == 6))
                throw new Exception("Nested routine attribute missing");
            Console.WriteLine("CLR attributes verified");
            """);
        await BuildAndRunContractProject(project, configuration, "CLR attributes verified");
    }
}
