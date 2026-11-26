// [Documentation](../../../docs/pascal/language/functions/parameters.md)
using Fpas.Compiler.Tests.Support;
using System.Runtime.Loader;

namespace Fpas.Compiler.Tests.Routines;

/// <summary>Checks arrays passed to imported Span&lt;T&gt; and ReadOnlySpan&lt;T&gt; parameters.</summary>
public sealed class SpanArgumentTests
{
    /// <summary>Executes span-only APIs, inference, overload preference and a later awaited argument.</summary>
    [Fact]
    public async Task Arrays_are_passed_to_imported_span_parameters()
    {
        var result = TestCompilation.Compile("""
            unit Spans;
            uses System as Sys; uses System.IO as IO; uses System.Buffers.Binary as Binary; uses System.Threading.Tasks as Tasks;

            async function Later(Value: int): Tasks.Task of int;
            begin
              await Tasks.Task.Delay(1);
              return Value;
            end function;

            public function Run(): string;
            begin
              const Header: array of byte := [$00, $00, $01, $02];
              const Stream: IO.MemoryStream := new IO.MemoryStream();
              Stream.Write(Header);
              return Binary.BinaryPrimitives.ReadUInt32BigEndian(Header).ToString() + ' ' + Stream.Length.ToString() + ' '
                + Sys.MemoryExtensions.IndexOf of int([4, 5, 6], 5).ToString() + ' ' + Sys.MemoryExtensions.IndexOf([7, 8, 9], 9).ToString() + ' '
                + new Sys.String([#120, #121]);
            end function;

            public async function Mixed(Data: array of byte): Tasks.Task of string;
            begin
              return Sys.Convert.ToHexString(Data) + ' ' + Sys.BitConverter.ToInt32(Data, await Later(0)).ToString();
            end function;
            end unit;
            """, CompilationTarget.Library);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        var context = new AssemblyLoadContext("span-arguments", isCollectible: true);
        try
        {
            context.LoadFromAssemblyPath(typeof(Core.RecordAttribute).Assembly.Location);
            var owner = context.LoadFromStream(new MemoryStream(result.Program!.AssemblyImage)).GetType("Spans")!;
            Assert.Equal("258 4 1 2 xy", owner.GetMethod("Run")!.Invoke(null, []));
            var mixed = (Task<string>)owner.GetMethod("Mixed")!.Invoke(null, [new byte[] { 0, 0, 1, 2 }])!;
            Assert.Equal("00000102 33619968", await mixed);
        }
        finally { context.Unload(); }
    }

    /// <summary>Rejects spans outside by-value call parameters and covariant mutable spans.</summary>
    /// <param name="body">Statements of the program body.</param>
    /// <param name="code">Expected diagnostic code.</param>
    /// <param name="message">Expected message fragment.</param>
    [Theory]
    [InlineData("discard Sys.MemoryExtensions.AsSpan of int(new int[2]);", "FP3005", "byref-like type")]
    [InlineData("const S: Sys.Span of int := new Sys.Span of int();", "FP3005", "Pass an array")]
    [InlineData("const F: function(array of byte): uint := Binary.BinaryPrimitives.ReadUInt32BigEndian;", "FP3005", "byref-like type")]
    [InlineData("const Items: array of string := ['a']; Sys.MemoryExtensions.Reverse of object(Items);", "FP3003", "exact array element type")]
    public void Spans_stay_limited_to_call_arguments(string body, string code, string message)
    {
        var result = TestCompilation.CompileAgainstFramework(
            "program Bad; uses System as Sys; uses System.Buffers.Binary as Binary; begin " + body + " end; end program;");
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal(code, diagnostic.Code);
        Assert.Contains(message, diagnostic.Message);
        Assert.Null(result.Program);
    }
}
