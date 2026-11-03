using Fpas.Tests.Support;

namespace Fpas.IntegrationTests.Build;

public sealed partial class MsBuildTests
{
    [Fact]
    public async Task Exported_overloads_resolve_in_CSharp_and_another_FPAS_unit()
    {
        using var project = sdk.CreateProject();
        File.WriteAllText(project.File("Game/Bridge.fpas"), """
            unit Demo.Bridge;
            public function Pick(Value: int): string; begin return 'int'; end function;
            public function Pick(Value: double): string; begin return 'double'; end function;
            public function Pick<T>(Value: T): string; begin return 'generic'; end function;
            public function Optional(Value: int := 1): string; begin return 'one'; end function;
            public function Optional(First: string; Second: int := 2): string; begin return First; end function;
            private function Pick(Value: bool): string; begin return 'private'; end function;
            end unit;
            """);
        File.WriteAllText(project.File("Game/Consumer.fpas"), """
            unit Demo.Consumer;
            uses Demo.Bridge as B;
            public function Run(): bool;
            begin
              const Callback: function(int): string := B.Pick;
              return B.Pick(1) = 'int' and B.Pick('x') = 'generic' and B.Pick(true) = 'generic' and
                Callback(2) = 'int' and B.Optional() = 'one' and B.Optional(First := 'named') = 'named';
            end function;
            end unit;
            """);
        File.WriteAllText(project.File("Host/Program.cs"), """
            using Demo;
            Func<int, string> callback = Bridge.Pick;
            if (!Consumer.Run() || Bridge.Pick(1) != "int" || Bridge.Pick(1.0) != "double" || Bridge.Pick("x") != "generic" || callback(2) != "int")
                throw new Exception("Wrong overload resolution.");
            Console.WriteLine("Overloads checked");
            """);
        await BuildAndRunContractProject(project, "Debug", "Overloads checked");
    }
}
