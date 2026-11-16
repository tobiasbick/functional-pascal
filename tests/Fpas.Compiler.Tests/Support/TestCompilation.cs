using Fpas.Tests.Support;

namespace Fpas.Compiler.Tests.Support;

internal static class TestCompilation
{
    /// <summary>Framework reference assemblies plus the FPAS runtime support assembly.</summary>
    internal static string[] References => [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location];

    internal static CompilationResult Compile(string source, CompilationTarget target = CompilationTarget.Exe)
        => FpasCompiler.Compile(new([new("records.fpas", source)], "Records", References, target));

    /// <summary>Compiles <c>example.fpas</c> against framework reference assemblies only.</summary>
    /// <param name="source">Program source.</param>
    /// <returns>The compilation result.</returns>
    internal static CompilationResult CompileAgainstFramework(string source)
        => FpasCompiler.Compile(new([new("example.fpas", source)], "Example", TestEnvironment.References));
}
