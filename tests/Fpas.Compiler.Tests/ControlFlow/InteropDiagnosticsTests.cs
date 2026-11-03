namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class InteropDiagnosticsTests
{
    [Theory]
    [InlineData("const X: string := null;", "FP3013")]
    [InlineData("const X: string? := null; Console.WriteLine(X.Length);", "FP3013")]
    [InlineData("const X: string? := Console.ReadLine(); const Y: string := cast<string>(X);", "FP3013")]
    [InlineData("const N: int? := null; const X: int := N;", "FP3013")]
    [InlineData("const X: string? := Console.ReadLine(); while true do Console.WriteLine(X.Length); X := null; end while;", "FP3013")]
    [InlineData("var X: string? := 'known'; while true do Console.WriteLine(X.Length); X := null; end while;", "FP3013")]
    [InlineData("var X: string? := 'known'; for I: int := 1 to 2 do Console.WriteLine(X.Length); X := null; end for;", "FP3013")]
    [InlineData("var X: string? := 'known'; if Console.ReadLine() = null then X := null; end if; Console.WriteLine(X.Length);", "FP3013")]
    [InlineData("const X: IO.StringReader? := null; discard X.ReadToEnd();", "FP3013")]
    [InlineData("const X: IO.TextReader := new IO.TextReader();", "FP3005")]
    [InlineData("const X: IO.StringReader? := new IO.StringReader?('text');", "FP3005")]
    [InlineData("const X: Sys.Void := new Sys.Void();", "FP3005")]
    [InlineData("const X: Sys.Runtime.CompilerServices.DefaultInterpolatedStringHandler := new Sys.Runtime.CompilerServices.DefaultInterpolatedStringHandler();", "FP3005")]
    [InlineData("using const X: string := 'text';", "FP3016")]
    [InlineData("using var X: IO.StringReader := new IO.StringReader('text');", "FP2001")]
    [InlineData("const D: Sys.DateTime := new Sys.DateTime(); D.Year := 1;", "FP3015")]
    [InlineData("try Console.WriteLine('x'); end try;", "FP2003")]
    [InlineData("try raise new Sys.Exception(); catch X: string do end try;", "FP3017")]
    [InlineData("try raise new Sys.Exception(); catch X: Sys.Exception do catch Y: Sys.InvalidOperationException do end try;", "FP3017")]
    [InlineData("try raise new Sys.Exception(); catch X: Sys.Exception? do end try;", "FP3017")]
    [InlineData("raise;", "FP3017")]
    [InlineData("raise 'wrong';", "FP3002")]
    [InlineData("try Console.WriteLine('x'); finally return; end try;", "FP3017")]
    [InlineData("while true do try break; finally continue; end try; end while;", "FP3017")]
    [InlineData("try raise new Sys.Exception(); catch Failure: Sys.Exception do end try; Console.WriteLine(Failure.Message);", "FP3001")]
    public void Unsafe_or_unsupported_interop_is_diagnosed(string body, string code)
    {
        var result = DiagnosticsTests.Compile("program Bad; uses System as Sys, System.IO as IO, System.Console as Console; begin " + body + " end; end program;");
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }
}
