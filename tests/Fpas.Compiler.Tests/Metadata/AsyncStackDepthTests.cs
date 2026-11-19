// [Documentation](../../../docs/pascal/language/concurrency/async-functions.md)
using Fpas.Compiler.Tests.Support;
using System.Reflection;
using System.Runtime.Loader;

namespace Fpas.Compiler.Tests.Metadata;

/// <summary>Verifies stack bounds for nested ordered arguments in an async body.</summary>
public sealed class AsyncStackDepthTests
{
    /// <summary>Executes an awaited call whose named/default arguments need nested hoisted stores.</summary>
    /// <returns>The compilation and CLR execution assertions.</returns>
    [Fact]
    public async Task Named_constructor_arguments_inside_a_configured_await_have_sufficient_stack()
    {
        const string source = """
            unit TaskStack;
            uses System.Threading.Tasks as Tasks;
            type Options = record
              public A: int := 1;
              public B: int := 2;
              public C: int := 3;
              public D: int := 4;
              public E: int := 5;
              public F: int := 6;
              public G: int := 7;
              public H: int := 8;
              public I: int := 9;
            end record;
            type Client = record
              public function Read(Value: Options): Tasks.Task<int>;
              begin return Tasks.Task.FromResult<int>(Value.I); end function;
            end record;
            public async function Run(): Tasks.Task<int>;
            begin
              const Session: Client := new Client();
              return await Session.Read(new Options(I := 42)).ConfigureAwait(false);
            end function;
            end unit;
            """;
        var result = FpasCompiler.Compile(new([new("stack.fpas", source)], "TaskStack",
            TestCompilation.References, CompilationTarget.Library));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var context = new AssemblyLoadContext("async-stack", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(result.Program!.AssemblyImage));
            var run = assembly.GetType("TaskStack")!.GetMethod("Run", BindingFlags.Static | BindingFlags.Public)!;
            Assert.Equal(42, await (Task<int>)run.Invoke(null, null)!);
        }
        finally
        {
            context.Unload();
        }
    }
}
