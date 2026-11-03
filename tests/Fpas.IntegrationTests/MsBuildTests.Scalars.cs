using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Data_domains_example_builds_and_runs()
    {
        using var project = sdk.CreateProject(); Directory.CreateDirectory(project.File("Domains"));
        foreach (var source in Directory.GetFiles(Path.Combine(AppContext.BaseDirectory, "Fixtures", "data-domains")))
            File.WriteAllText(project.File("Domains/" + Path.GetFileName(source)), File.ReadAllText(source).Replace("0.1.0-dev", sdk.Version, StringComparison.Ordinal));
        var build = await sdk.Run(project, "build", "Domains/DataDomains.fpasproj");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await ProcessRunner.RunAsync([project.File("Domains/bin/Debug/net10.0/DataDomains.dll")], workingDirectory: project.Path,
            environment: new Dictionary<string, string?> { ["DOTNET_SYSTEM_GLOBALIZATION_INVARIANT"] = "1" });
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal(string.Join(Environment.NewLine, "42", "Ada", "Online", "19.95", "system", "1", ""), run.Output);
    }

    [Fact]
    public async Task Scalar_contracts_round_trip_between_CSharp_and_separate_Fpas_assemblies()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/SharedApi.cs"), """
            namespace Shared;
            public enum External : uint { High = uint.MaxValue }
            public static class Numbers
            {
                public const decimal Price = 1.25M;
                public static decimal Add(decimal value, decimal extra = 0.75M) => value + extra;
                public static ulong Maximum(ulong value = ulong.MaxValue) => value;
                public static nint Native(nint value) => value;
                public static nint NativeDefault(nint value = 1) => value;
                public static nuint UnsignedDefault(nuint value = uint.MaxValue) => value;
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Models;
            public type Count = ulong;
            public type MaybeText = string?;
            public type Texts = string?[];
            public type Box<T> = record Value: T; end record;
            public type CountBox = Box<Count>;
            public type Direction = enum : byte North; East = 4; West; end enum;
            public type Huge = enum : ulong Max = 18446744073709551615UL; end enum;
            public type UserId = distinct long;
            public type Name = distinct string;
            public type Money = distinct decimal;
            public type Port = ushort range 1..65535;
            public type Negative = long range -9223372036854775808L..-1L;
            public type High = ulong range 18446744073709551610UL..18446744073709551615UL;
            public function Amount(Value: decimal := 1.25M): decimal; begin return Value; end function;
            public function Largest(Value: Count := 18446744073709551615UL): Count; begin return Value; end function;
            public function Echo(Value: MaybeText): MaybeText; begin return Value; end function;
            public function EchoTexts(Value: Texts): Texts; begin return Value; end function;
            public function Boxed(Value: Count): CountBox; begin return new CountBox(Value); end function;
            public function Native(Value: nint; Other: nuint): nint; begin return Value + cast<nint>(Other); end function;
            public function NativeDefault(Value: nint := 0): nint; begin return Value; end function;
            public function NativeOne(Value: nint := 1): nint; begin return Value; end function;
            public function CharFloat(Value: float := #65): float; begin return Value; end function;
            public function CharDecimal(Value: decimal := #65): decimal; begin return Value; end function;
            public function Selected(Value: Direction := Direction.West): Direction; begin return Value; end function;
            public function Small(A: sbyte; B: byte; C: short; D: ushort; E: float): double; begin return A + B + C + D + E; end function;
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
            uses Demo.Models as M, Shared as S;
            public function Maximum(): M.Count; begin return M.Largest(); end function;
            public function Text(Value: M.MaybeText): M.MaybeText; begin return M.Echo(Value); end function;
            public function Texts(Value: M.Texts): M.Texts; begin return M.EchoTexts(Value); end function;
            public function Boxed(): M.CountBox; begin return M.Boxed(18446744073709551615UL); end function;
            public function Id(Value: long): M.UserId; begin return new M.UserId(Value); end function;
            public function IdValue(Value: M.UserId): long; begin return cast<long>(Value); end function;
            public function Port(Value: ushort): M.Port; begin return cast<M.Port>(Value); end function;
            public function PortValue(Value: M.Port): int; begin return Value; end function;
            public function Bounds(): int; begin return M.Port.LowerBound + M.Port.UpperBound; end function;
            public function Price(): decimal; begin return S.Numbers.Add(S.Numbers.Price) + M.Amount() + S.Numbers.Price - S.Numbers.Price; end function;
            public function Foreign(): uint; begin return cast<uint>(S.External.High); end function;
            public function Combine(): M.Direction; begin return M.Direction.East or M.Direction.West; end function;
            public function Native(): nint;
            begin
              discard S.Numbers.UnsignedDefault();
              return S.Numbers.Native(cast<nint>(40)) + S.Numbers.NativeDefault() + M.NativeOne() + M.NativeDefault();
            end function;
            end unit;
            """);
        var hostPath = project.File("Host/Host.csproj"); var host = XDocument.Load(hostPath);
        host.Root!.Add(new XElement("ItemGroup", new XElement("ProjectReference", new XAttribute("Include", "../Consumer/Consumer.fpasproj")))); host.Save(hostPath);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            if (Consumer.Maximum() != ulong.MaxValue || Models.Largest() != ulong.MaxValue) throw new Exception("Unsigned defaults");
            if (Models.Amount() != 1.25M || Consumer.Price() != 3.25M) throw new Exception("Decimal metadata");
            if (Consumer.Foreign() != uint.MaxValue || Consumer.Combine() != Models.Direction.West) throw new Exception("Enum import");
            if (!typeof(Models.Direction).IsEnum || Enum.GetUnderlyingType(typeof(Models.Direction)) != typeof(byte) || (ulong)Models.Huge.Max != ulong.MaxValue) throw new Exception("Enum representation");
            if (Models.Selected() != Models.Direction.West || Models.NativeDefault() != 0 || Models.NativeOne() != 1) throw new Exception("Optional enum/native parameters");
            if (Models.CharFloat() != 65F || Models.CharDecimal() != 65M) throw new Exception("Character constant conversions");
            if (Consumer.Text(null) != null || Consumer.Texts(new string?[] { null, "yes" })[0] != null) throw new Exception("Alias nullability");
            if (Consumer.Boxed().Value != ulong.MaxValue || Consumer.Boxed().GetType() != typeof(Models.Box<ulong>)) throw new Exception("Closed generic alias");
            if (Consumer.Native() != 42 || Models.Native(40, 2) != 42 || Models.Small(-1, 2, -3, 4, 0.5F) != 2.5) throw new Exception("Primitive ABI");
            var id = Consumer.Id(42); var other = new Models.UserId(43);
            if (Consumer.IdValue(id) != 42 || id.Value != 42 || (long)(Models.UserId)42L != 42 || id == other || id >= other || !id.Equals((object)new Models.UserId(42))) throw new Exception("Distinct interop");
            if (new HashSet<Models.UserId> { id, new(42) }.Count != 1 || default(Models.UserId).Value != 0 || (object)id is long) throw new Exception("Distinct identity");
            if (!(new Models.Money(1M) < new Models.Money(2M)) || !(new Models.Name("a") < new Models.Name("z"))) throw new Exception("Ordered domains");
            if (new Models.Name("a") != new Models.Name("a") || new Models.Name("a").Equals((object)new Models.Name("b"))) throw new Exception("Reference equality");
            try { _ = new Models.Name(null!); throw new Exception("Null accepted"); } catch (ArgumentNullException) { }
            var port = Consumer.Port(80);
            if (Consumer.PortValue(port) != 80 || Consumer.Bounds() != 65536 || (ushort)port != 80 || new Models.Port(80) != port) throw new Exception("Subrange import");
            if (default(Models.Port).Value != 1 || (new Models.Port[1])[0].Value != 1 || Create<Models.Port>().Value != 1) throw new Exception("Subrange defaults");
            if (default(Models.Negative).Value != long.MinValue || new Models.Negative(-1).Value != -1 || default(Models.High).Value != ulong.MaxValue - 5 || new Models.High(ulong.MaxValue).Value != ulong.MaxValue) throw new Exception("Offset boundaries");
            try { _ = (Models.Port)(ushort)0; throw new Exception("Invalid range accepted"); } catch (ArgumentOutOfRangeException) { }
            if (typeof(Models).GetNestedTypes().Any(t => t.Name is "Count" or "MaybeText" or "Texts")) throw new Exception("Alias CLR identity");
            Console.WriteLine("Scalar interop verified");
            static T Create<T>() where T : new() => new T();
            """);
        var build = await sdk.Run(project, "build", "Host/Host.csproj", "-p:TreatWarningsAsErrors=true");
        Assert.True(build.ExitCode == 0, build.Output + build.Error);
        var run = await sdk.Run(project, "Host/bin/Debug/net10.0/Host.dll");
        Assert.True(run.ExitCode == 0, run.Output + run.Error);
        Assert.Equal("Scalar interop verified" + Environment.NewLine, run.Output);
        File.WriteAllText(project.File("Host/Invalid.cs"), "public static class Invalid { public static string Read() => Demo.Consumer.Text(null); }");
        var nullable = await sdk.Run(project, "build", "Host/Host.csproj", "--no-restore", "-p:TreatWarningsAsErrors=true");
        Assert.NotEqual(0, nullable.ExitCode); Assert.Contains("CS8603", nullable.Output);
    }
}
