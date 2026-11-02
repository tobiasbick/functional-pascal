using Fpas.Tests.Support;

namespace Fpas.IntegrationTests;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Records_implement_inherited_closed_generic_properties_methods_modes_and_defaults()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/Interfaces.cs"), """
            namespace Demo;
            public interface IName { string Name { get; } }
            public interface IBase { int Get(); }
            public interface IDerived : IBase { int IBase.Get() => 23; }
            public interface IDefault { int Get() => 17; }
            public interface IApi<T> : IName
            {
                T Read();
                T Read(T value);
                U Echo<U>(U value) where U : class;
                void Adjust(ref int value, out int previous, in int delta);
                int Default() => 17;
            }
            public static class Interfaces
            {
                public static string Call(IApi<string> value)
                {
                    var number = 1; var delta = 2; value.Adjust(ref number, out var before, in delta);
                    if (number != 3 || before != 1 || value.Default() != 17 || value.Echo("echo") != "echo") throw new Exception("Bad interface implementation.");
                    return value.Name + value.Read() + value.Read("other");
                }
            }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            uses Demo as D, System.Collections.Generic as Collections;
            public type Api<T> = record implements D.IApi<T>
              Name: string;
              Value: T;
              function Read(): T; begin return self.Value; end function;
              function Read(Other: T): T; begin return Other; end function;
              function Echo<U>(Value: U): U; where U: class; begin return Value; end function;
              procedure Adjust(ref Value: int; out Before: int; in Step: int);
              begin Before := Value; Value := Value + Step; end procedure;
            end record;
            public type Descending = record implements Collections.IComparer<int>
              function Compare(Left: int; Right: int): int; begin return Right - Left; end function;
            end record;
            public type Default = record implements D.IDerived end record;
            public type Override = record implements D.IDefault
              function Get(): int; begin return 29; end function;
            end record;
            public function Run(): string;
            begin
              const Value: D.IApi<string> := new Api<string>('name', 'value');
              const Numbers: int[] := [1, 3, 2];
              const List: Collections.List<int> := new Collections.List<int>(Numbers);
              List.Sort(new Descending());
              if List[0] <> 3 then return 'wrong'; end if;
              return D.Interfaces.Call(Value);
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            var value = new Bridge.Api<string>("name", "value");
            if (Interfaces.Call(value) != "namevalueother" || Bridge.Run() != "namevalueother") throw new Exception("Wrong interface dispatch.");
            IApi<string> api = value;
            if (!ReferenceEquals(api, value) || api.Default() != 17) throw new Exception("Wrong record identity/default implementation.");
            if (((IBase)new Bridge.Default()).Get() != 23 || ((IDefault)new Bridge.Override()).Get() != 29) throw new Exception("Wrong most-specific default implementation.");
            Console.WriteLine("Interfaces checked");
            """);
        await BuildAndRunContractProject(project, "Debug", "Interfaces checked");
    }

    [Theory]
    [InlineData("IProperty", "Value: string?;")]
    [InlineData("IParameter", "procedure Write(Value: string); begin end procedure;")]
    [InlineData("IRef", "procedure Change(ref Value: string); begin end procedure;")]
    [InlineData("IList", "function Read(): Collections.List<string?>; begin return new Collections.List<string?>(); end function;")]
    [InlineData("ISetter", "Value: int;")]
    [InlineData("IIndex", "")]
    [InlineData("IEvent", "")]
    [InlineData("IStatic", "")]
    [InlineData("IConstraint", "function Echo<T>(Value: T): T; begin return Value; end function;")]
    [InlineData("IGenericNullable", "function Read<T>(): T?; where T: class; begin return null; end function;")]
    [InlineData("IDiamond", "")]
    public async Task Incompatible_nullability_and_unsupported_interface_members_are_rejected(string contract, string members)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/InvalidInterfaces.cs"), """
            namespace Demo;
            public interface IProperty { string Value { get; } }
            public interface IParameter { void Write(string? value); }
            public interface IRef { void Change(ref string? value); }
            public interface IList { List<string> Read(); }
            public interface ISetter { int Value { get; set; } }
            public interface IIndex { int this[int index] { get; } }
            public interface IEvent { event Action Changed; }
            public interface IStatic { static abstract int Read(); }
            public interface IConstraint { T Echo<T>(T value) where T : class; }
            public interface IGenericNullable { T Read<T>() where T : class; }
            public interface IRoot { int Read() => 1; }
            public interface ILeft : IRoot { int IRoot.Read() => 2; }
            public interface IRight : IRoot { int IRoot.Read() => 3; }
            public interface IDiamond : ILeft, IRight { }
            """);
        File.WriteAllText(project.File("Game/Bridge.fpas"), $$"""
            unit Demo.Bridge;
            uses Demo as D, System.Collections.Generic as Collections;
            public type Invalid = record implements D.{{contract}} {{members}} end record;
            end unit;
            """);
        var result = await sdk.Run(project, "build", "Game/Game.fpasproj");
        Assert.NotEqual(0, result.ExitCode); Assert.Contains("FP3029", result.Output + result.Error);
    }
}
