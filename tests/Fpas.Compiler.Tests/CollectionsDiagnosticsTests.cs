namespace Fpas.Compiler.Tests;

public sealed class CollectionsDiagnosticsTests
{
    [Theory]
    [InlineData("const X: string[] := new string[1];", "FP3013")]
    [InlineData("const X: string[,] := new string[1, 2];", "FP3013")]
    [InlineData("const X: int[] := []; X[0, 1] := 2;", "FP3002")]
    [InlineData("const X: int[] := []; X[true] := 2;", "FP3002")]
    [InlineData("const X: int[] := []; X[0] := 'wrong';", "FP3002")]
    [InlineData("const X: int[]? := null; discard X[0];", "FP3013")]
    [InlineData("const X: int[] := []; X.Length := 1;", "FP3015")]
    [InlineData("const X: string := 'text'; X[0] := #65;", "FP3015")]
    [InlineData("discard [];", "FP3002")]
    [InlineData("const X: char := #65536;", "FP3002")]
    [InlineData("const X: C.List<string> := new C.List<string>(); X.Add(null);", "FP3013")]
    [InlineData("const X: C.List<string?> := new C.List<string?>(); const Y: C.List<string> := X;", "FP3002")]
    [InlineData("const X: C.List<string> := new C.List<string>(); const Y: C.List<string?> := X;", "FP3002")]
    [InlineData("const X: C.Dictionary<string?, int> := new C.Dictionary<string?, int>();", "FP3018")]
    [InlineData("const X: C.IEnumerable<string?> := new C.List<string?>(); for Item: string in X do end for;", "FP3002")]
    [InlineData("for Item: string in [1, 2] do end for;", "FP3002")]
    [InlineData("for Item: int in [1] do Item := 2; end for;", "FP3009")]
    [InlineData("for Item in [1] do end for;", "FP2002")]
    [InlineData("for Item: int in 42 do end for;", "FP3002")]
    [InlineData("for Item: object? in null do end for;", "FP3002")]
    [InlineData("const X: string[] := ['known']; const Y: object?[] := X;", "FP3002")]
    [InlineData("args := [];", "FP3009")]
    [InlineData("const args: string[] := [];", "FP3004")]
    [InlineData("var X: string? := 'known'; for Item: int in [1, 2] do discard X.Length; X := null; end for;", "FP3013")]
    public void Invalid_collection_operations_are_diagnosed(string body, string code)
    {
        var result = DiagnosticsTests.Compile("program Bad; uses System.Collections.Generic as C; begin " + body + " end; end program;");
        Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
        Assert.Null(result.Program);
    }

    [Fact]
    public void Helpers_do_not_implicitly_receive_program_arguments()
    {
        var result = DiagnosticsTests.Compile("program Bad; function Count(): int; begin return args.Length; end function; begin end; end program;");
        Assert.Equal("FP3001", Assert.Single(result.Diagnostics).Code);
    }
}
