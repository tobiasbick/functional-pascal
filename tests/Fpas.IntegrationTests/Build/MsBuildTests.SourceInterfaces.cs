using System.Xml.Linq;
using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Theory]
    [InlineData("Debug")]
    [InlineData("Release")]
    public async Task Source_interfaces_dispatch_across_Fpas_CSharp_and_separate_assemblies(string configuration)
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Shared/DefaultApi.cs"), """
            namespace Demo;
            public interface IDefaultApi { int Default() => 17; }
            public interface IOverrideApi : IDefaultApi { int IDefaultApi.Default() => 19; }
            public interface IExceptionApi { U Exception<U>(U value) where U : Exception; }
            """);
        File.WriteAllText(project.File("Game/Contracts.fpas"), """
            unit Demo.Contracts;
            uses System as Sys, Demo as D, System.Threading.Tasks as Tasks, System.ComponentModel as Metadata;
            public var Disposals: int := 0;
            [Metadata.DescriptionAttribute('source interface')]
            public type IRoot<T> = interface
              [Metadata.DescriptionAttribute('value property')]
              Value: T;
              [Metadata.DescriptionAttribute('read method')]
              function Read(): T;
            end interface;
            public type ILeft<T> = interface extends IRoot<T> end interface;
            public type IRight<T> = interface extends IRoot<T> end interface;
            public type IOther<T> = interface Value: T; function Read(): T; end interface;
            public type IApi<T> =
            where T: class;
            interface extends ILeft<T>, IRight<T>, D.IOverrideApi, D.IExceptionApi, Sys.IDisposable
              Name: string;
              function Read(Other: T): T;
              function Echo<U>(Value: U): U;
              where U: class;
              function Narrow<U>(Value: U): U;
              where U: T;
              procedure Adjust(ref Value: int; out Before: int; in Step: int);
              function Sum(Base: int := 3; params Values: array of int): int;
              function Optional(Value: string?): string?;
              function ReadAsync(): Tasks.Task<T>;
            end interface;
            public type Api<T> =
            where T: class;
            record implements IApi<T>, IOther<T>
              public Name: string;
              public Value: T;
              public function Read(): T; begin return self.Value; end function;
              public function Read(Other: T): T;
              requires Other <> null;
              begin return Other; end function;
              public function Echo<U>(Value: U): U; where U: class; begin return Value; end function;
              public function Narrow<U>(Value: U): U; where U: T; begin return Value; end function;
              public function Exception<U>(Value: U): U; where U: Sys.Exception; begin return Value; end function;
              public procedure Adjust(ref Value: int; out Before: int; in Step: int);
              begin Before := Value; Value := Value + Step; end procedure;
              public function Sum(Base: int := 3; params Values: array of int): int;
              begin var Total: int := Base; for Value: int in Values do Total := Total + Value; end for; return Total; end function;
              public function Optional(Value: string?): string?; begin return Value; end function;
              public async function ReadAsync(): Tasks.Task<T>; begin return self.Value; end function;
              public procedure Dispose(); begin Disposals := Disposals + 1; end procedure;
            end record;
            public function Accept(Value: IApi<string>): string;
            begin
              var Number: int := 1; var Before: int := 0; const Step: int := 2;
              Value.Adjust(ref Number, out Before, in Step);
              if Number <> 3 or Before <> 1 or Value.Sum(Values := [4, 5]) <> 12 or
                Value.Default() <> 19 or Value.Optional(null) <> null then
                raise new Sys.InvalidOperationException('Incorrect interface operations');
              end if;
              const Read: function(): string := Value.Read;
              return Value.Name + Value.Value + Read() + Value.Read(Other := 'other') + Value.Echo('echo');
            end function;
            public function Constrained<T>(Value: T): string;
            where T: IApi<string>;
            begin return Value.Read(); end function;
            public function Infer<T>(Value: IRoot<T>): T; begin return Value.Read(); end function;
            public function Local(): string;
            begin
              const Value: IApi<string> := new Api<string>('fpas', 'value');
              if Infer(Value) <> 'value' then raise new Sys.InvalidOperationException('Interface inference failed'); end if;
              return Accept(Value);
            end function;
            end unit;
            """);
        Directory.CreateDirectory(project.File("Consumer"));
        File.WriteAllText(project.File("Consumer/Consumer.fpasproj"), $$"""
            <Project Sdk="Fpas.NET.Sdk/{{sdk.Version}}">
              <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
              <ItemGroup><ProjectReference Include="../Game/Game.fpasproj" /></ItemGroup>
            </Project>
            """);
        File.WriteAllText(project.File("Consumer/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Contracts as C, System as Sys, System.Threading.Tasks as Tasks;
            public type Local = record implements C.IRoot<string>
              public Value: string;
              public function Read(): string; begin return self.Value; end function;
            end record;
            public type External<T> = where T: class;
            record implements C.IApi<T>
              public Name: string; public Value: T;
              public function Read(): T; begin return self.Value; end function;
              public function Read(Other: T): T; begin return Other; end function;
              public function Echo<U>(Value: U): U; where U: class; begin return Value; end function;
              public function Narrow<U>(Value: U): U; where U: T; begin return Value; end function;
              public function Exception<U>(Value: U): U; where U: Sys.Exception; begin return Value; end function;
              public procedure Adjust(ref Value: int; out Before: int; in Step: int); begin Before := Value; Value := Value + Step; end procedure;
              public function Sum(Base: int := 3; params Values: array of int): int; begin return Base; end function;
              public function Optional(Value: string?): string?; begin return Value; end function;
              public function ReadAsync(): Tasks.Task<T>; begin return Tasks.Task.FromResult(self.Value); end function;
              public procedure Dispose(); begin end procedure;
            end record;
            public function Accept(Value: C.IApi<string>): string;
            begin return Value.Read() + Value.Echo('external') + Value.ReadAsync().GetAwaiter().GetResult(); end function;
            public function Run(): string;
            begin const Value: C.IRoot<string> := new Local('consumer'); return Value.Value + Value.Read(); end function;
            end unit;
            """);
        var host = project.File("Host/Host.csproj");
        File.WriteAllText(host, File.ReadAllText(host).Replace("../Game/Game.fpasproj", "../Consumer/Consumer.fpasproj", StringComparison.Ordinal));
        var solution = XDocument.Load(project.File("Game.slnx"));
        solution.Root!.Add(new XElement("Project", new XAttribute("Path", "Consumer/Consumer.fpasproj")));
        solution.Save(project.File("Game.slnx"));
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            using System.Reflection;
            var fpas = new Contracts.Api<string>("fpas", "value");
            Contracts.IApi<string> api = fpas;
            if (Contracts.Local() != "fpasvaluevalueotherecho" || Contracts.Accept(api) != "fpasvaluevalueotherecho") throw new Exception("Local dispatch failed.");
            if (Contracts.Constrained(fpas) != "value" || await api.ReadAsync() != "value" || api.Sum(Values: [4, 5]) != 12) throw new Exception("Generic dispatch failed.");
            if (((Contracts.IOther<string>)fpas).Read() != "value" || ((Contracts.IOther<string>)fpas).Value != "value") throw new Exception("Shared slots failed.");
            var external = new Consumer.External<string>("external", "value");
            Contracts.IApi<string> imported = external;
            if (imported.Narrow("narrow") != "narrow" || api.Narrow("local") != "local" || imported.Exception(new Exception("bound")).Message != "bound") throw new Exception("Constraint substitution failed.");
            if (Consumer.Accept(api) != "valueexternalvalue" || Consumer.Run() != "consumerconsumer") throw new Exception("External FPAS dispatch failed.");
            var csharp = new CSharpApi<string>("value");
            if (Contracts.Accept(csharp) != "csharpvaluevalueotherecho" || Consumer.Accept(csharp) != "valueexternalvalue") throw new Exception("C# implementation failed.");
            try { api.Read(null!); throw new Exception("Contract did not execute."); } catch (InvalidOperationException) { }
            using (api) { }
            if (Contracts.Disposals != 1) throw new Exception("Inherited Dispose failed.");
            var contract = typeof(Contracts.IApi<>);
            if (!contract.IsInterface || contract.GetGenericArguments()[0].GenericParameterAttributes.HasFlag(GenericParameterAttributes.Covariant)) throw new Exception("Wrong interface metadata.");
            var adjust = contract.GetMethod("Adjust")!.GetParameters();
            if (!adjust[0].ParameterType.IsByRef || !adjust[1].IsOut || !adjust[2].IsIn) throw new Exception("Wrong parameter modes.");
            if (contract.GetMethod("ReadAsync")!.GetMethodBody() is not null) throw new Exception("Abstract method has a body.");
            var root = typeof(Contracts.IRoot<>);
            if (root.GetCustomAttribute<System.ComponentModel.DescriptionAttribute>()?.Description != "source interface" ||
                root.GetProperty("Value")!.GetCustomAttribute<System.ComponentModel.DescriptionAttribute>()?.Description != "value property" ||
                root.GetMethod("Read")!.GetCustomAttribute<System.ComponentModel.DescriptionAttribute>()?.Description != "read method") throw new Exception("Source attributes were lost.");
            var nullable = new NullabilityInfoContext();
            if (nullable.Create(contract.GetMethod("Optional")!.ReturnParameter).ReadState != NullabilityState.Nullable ||
                nullable.Create(contract.GetMethod("Optional")!.GetParameters()[0]).ReadState != NullabilityState.Nullable ||
                nullable.Create(contract.GetProperty("Name")!).ReadState != NullabilityState.NotNull) throw new Exception("Nullable metadata was lost.");
            Console.WriteLine("Source interfaces checked");
            public sealed class CSharpApi<T>(T value) : Contracts.IApi<T> where T : class
            {
                public string Name => "csharp";
                public T Value => value;
                public T Read() => Value;
                public T Read(T other) => other;
                public U Echo<U>(U value) where U : class => value;
                public U Narrow<U>(U value) where U : T => value;
                public U Exception<U>(U value) where U : Exception => value;
                public void Adjust(ref int value, out int before, in int step) { before = value; value += step; }
                public int Sum(int Base = 3, params int[] Values) => Base + Values.Sum();
                public string? Optional(string? value) => value;
                public Task<T> ReadAsync() => Task.FromResult(Value);
                public void Dispose() { }
            }
            """);
        await BuildAndRunContractProject(project, configuration, "Source interfaces checked");
    }
}
