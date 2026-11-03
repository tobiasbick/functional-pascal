using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Support;

internal static class TestCompilation
{
    internal static CompilationResult Compile(string source, CompilationTarget target = CompilationTarget.Exe)
        => FpasCompiler.Compile(new([new("records.fpas", source)], "Records",
            [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location], target));
}
