using Fpas.Compiler.Tests.ControlFlow;
namespace Fpas.Compiler.Tests.Routines;

public sealed class GenericRoutineTests
{
    [Theory]
    [InlineData("function F<T, T>(X: T): T; begin return X; end function;", "FP3026")]
    [InlineData("function F<int>(X: int): int; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T?): T; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): string; begin return X.ToString() ?? ''; end function;", "FP3013")]
    [InlineData("function F<T>(): T; begin return new T(); end function;", "FP3018")]
    [InlineData("function F<T>(): T[]; begin return new T[1]; end function;", "FP3013")]
    [InlineData("function F<T>(X: T): object; begin return X; end function;", "FP3013")]
    [InlineData("function F<T>(X: T): T; where U: class; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): T; where T: class; where T: new(); begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): T; where T: struct, new(); begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): T; where T: new(), class; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): T; where T: T; begin return X; end function;", "FP3026")]
    [InlineData("function F<T, U>(X: T): T; where T: U; where U: T; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T): T; where T: object?; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(X: T; Y: T): T; begin return X + Y; end function;", "FP3002")]
    [InlineData("function F<T>(X: T; Y: T): T; where T: N.IAdditionOperators<T, T, T>; begin return X * Y; end function;", "FP3002")]
    [InlineData("function F<T>(X: T?): int; where T: class, N.IEqualityOperators<T, T, bool>; begin if X <> null then return X.GetHashCode(); else return 0; end if; end function;", "FP3013")]
    [InlineData("function F<T>(X: T): T; where T: Sys.IComparable<T>, Sys.IComparable<T>; begin return X; end function;", "FP3026")]
    [InlineData("function F<T>(Input: T): int; where T: class?; begin var Value: T := Input; const Change: procedure() := procedure() begin Value := Input; discard Value.ToString(); end procedure; return 0; end function;", "FP3013")]
    [InlineData("function F<T>(X: T): T; where T: string[]; begin return X; end function;", "FP3026")]
    public void Invalid_generic_declarations_and_unsafe_bodies_are_diagnosed(string declaration, string code)
    {
        var result = DiagnosticsTests.Compile("program Example; uses System as Sys, System.Numerics as N; " + declaration + " begin end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("discard Id(null);", "FP3026")]
    [InlineData("discard Id<int, int>(42);", "FP3026")]
    [InlineData("discard Id<int>('wrong');", "FP3003")]
    [InlineData("discard Ref<int>(42);", "FP3018")]
    [InlineData("discard Ref<string?>(null);", "FP3018")]
    [InlineData("discard Value<int?>(null);", "FP3018")]
    [InlineData("discard Create<string>();", "FP3018")]
    public void Explicit_arguments_and_constraints_are_checked(string body, string code)
    {
        var result = DiagnosticsTests.Compile("""
            program Example;
            function Id<T>(X: T): T; begin return X; end function;
            function Ref<T>(X: T): T; where T: class; begin return X; end function;
            function Value<T>(X: T): T; where T: struct; begin return X; end function;
            function Create<T>(): T; where T: new(); begin return new T(); end function;
            begin
            """ + " " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }
}
