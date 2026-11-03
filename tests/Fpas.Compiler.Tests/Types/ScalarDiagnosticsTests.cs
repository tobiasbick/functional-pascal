using Fpas.Compiler.Tests.Support;
namespace Fpas.Compiler.Tests.Types;

public sealed class ScalarDiagnosticsTests
{
    [Theory]
    [InlineData("type A = B; type B = A;", "", "FP3031")]
    [InlineData("type Box<T> = where T: class; record Value: T; end record; type Bad = Box<int>;", "", "FP3018")]
    [InlineData("type long = int;", "", "FP3031")]
    [InlineData("type R = enum end enum;", "", "FP3031")]
    [InlineData("type R = enum A; A; end enum;", "", "FP3031")]
    [InlineData("type R = enum : byte A = 255; B; end enum;", "", "FP3031")]
    [InlineData("type R = enum A = B; B = A; end enum;", "", "FP3031")]
    [InlineData("type R = int range 100..1;", "", "FP3031")]
    [InlineData("type R = int range 1..100;", "discard new R(101);", "FP3031")]
    [InlineData("type R = int range 1..100;", "discard unchecked(cast<R>(101));", "FP3031")]
    [InlineData("type R = distinct object;", "", "FP3031")]
    [InlineData("type R = distinct int?;", "", "FP3031")]
    [InlineData("type R = distinct int; type S = distinct R;", "", "FP3031")]
    [InlineData("type R = double range 0.0..1.0;", "", "FP3031")]
    [InlineData("type R = distinct int;", "const X: R := 1;", "FP3002")]
    [InlineData("type R = distinct int;", "const X: int := new R(1);", "FP3002")]
    [InlineData("type R = distinct int;", "discard new R(1) + new R(2);", "FP3002")]
    [InlineData("type R = distinct string;", "discard new R(null);", "FP3013")]
    [InlineData("type R = distinct string;", "const X: R? := null; discard cast<string>(X);", "FP3013")]
    [InlineData("type R = distinct string;", "const X: R? := null; discard X < new R('a');", "FP3013")]
    [InlineData("type R = distinct string;", "const X: string? := null; discard cast<R?>(X);", "FP3013")]
    [InlineData("type R = byte range 1..3;", "const X: R := new R(); discard case X of when 0: 0; else 1; end case;", "FP3024")]
    [InlineData("type R = int range 1..3;", "const X: R := new R(); discard case X of when 1..2: 0; end case;", "FP3025")]
    [InlineData("", "const X: ulong := 18446744073709551616UL;", "FP3002")]
    [InlineData("", "const X: long := 9223372036854775808L;", "FP3002")]
    [InlineData("", "const X: byte := 256;", "FP3002")]
    [InlineData("", "discard 1M + 1.0;", "FP3002")]
    [InlineData("", "discard 1UL + -1L;", "FP3002")]
    [InlineData("", "discard -1UL;", "FP3002")]
    [InlineData("", "discard $;", "FP1004")]
    [InlineData("", "discard 1__0;", "FP1003")]
    [InlineData("", "discard 1.0UL;", "FP1004")]
    [InlineData("", "discard #65536;", "FP3002")]
    [InlineData("procedure P(Value: byte := unchecked(cast<byte>(256M))); begin end procedure;", "", "FP3028")]
    [InlineData("procedure P(Value: nint := cast<nint>(4294967296L)); begin end procedure;", "", "FP3028")]
    public void Invalid_scalar_declarations_and_implicit_domain_crossings_are_rejected(string declaration, string body, string code)
    {
        var result = TestCompilation.Compile("program Example; " + declaration + " begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }
}
