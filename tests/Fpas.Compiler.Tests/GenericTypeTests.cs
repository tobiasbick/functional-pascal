namespace Fpas.Compiler.Tests;

public sealed class GenericTypeTests
{
    [Theory]
    [InlineData("type Box<T, T> = record Value: T; end record;", "", "FP3026")]
    [InlineData("type Box<T> = where U: class; record Value: T; end record;", "", "FP3026")]
    [InlineData("type Box<T> = record Value: T?; end record;", "", "FP3026")]
    [InlineData("type Box<T> = record Value: T; end record;", "const X: Box := new Box(1);", "FP3026")]
    [InlineData("type Box<T> = record Value: T; end record;", "const X: Box<int, string> := new Box<int, string>(1);", "FP3026")]
    [InlineData("type Box<T> = where T: class; record Value: T; end record;", "const X: Box<int> := new Box<int>(1);", "FP3018")]
    [InlineData("type Box<T> = where T: class; record Value: T; end record;", "const X: Box<string?> := new Box<string?>(null);", "FP3018")]
    [InlineData("type Box<T> = where T: struct; record Value: T; end record;", "const X: Box<int?> := new Box<int?>(null);", "FP3018")]
    [InlineData("type Box<T> = where T: class; record Value: T := null; end record;", "", "FP3013")]
    [InlineData("type Box<T> = record function F<T>(X: T): T; begin return X; end function; end record;", "", "FP3026")]
    [InlineData("type Box<T> = record function F<U>(X: U): U; where T: class; begin return X; end function; end record;", "", "FP3026")]
    [InlineData("type Tree<T> = union Empty; Leaf(Value: T); end union;", "discard Tree.Empty;", "FP3026")]
    [InlineData("type Tree<T> = union Empty; Leaf(Value: T); end union;", "discard Tree.Leaf(null);", "FP3026")]
    [InlineData("type Tree<T> = where T: class; union Empty; Leaf(Value: T); end union;", "discard Tree<int>.Leaf(1);", "FP3018")]
    [InlineData("type Tree<T> = union Empty; Leaf(Value: T); end union;", "const X: Tree<int> := Tree<int>.Leaf(1); discard case X of when Tree.Leaf(const V: string): 0; when Tree.Empty: 1; end case;", "FP3024")]
    [InlineData("type Box<T> = where T: class; record Value: T; end record; function F<U>(Value: U): Box<U>; begin return new Box<U>(Value); end function;", "", "FP3018")]
    [InlineData("type Box<T> = record Value: T; end record;", "const A: Box<int> := new Box<int>(1); const B: Box<string> := A;", "FP3002")]
    [InlineData("type Box<T> = record Value: T; end record;", "const A: Box<string> := new Box<string>('a'); const B: Box<object> := A;", "FP3002")]
    [InlineData("type Factory<T> = where T: new(); record Value: T; end record; type A = record Value: Factory<B>; end record; type B = record X: int; end record;", "", "FP3018")]
    public void Invalid_generic_types_are_diagnosed(string declaration, string body, string code)
    {
        var result = RecordTests.Compile("program Example; " + declaration + " begin " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Theory]
    [InlineData("public type Box<T> = where T: Hidden; record Value: T; end record;")]
    [InlineData("public type Choice<T> = where T: Hidden; union Item(Value: T); end union;")]
    public void Public_type_constraints_cannot_expose_private_types(string declaration)
    {
        var result = RecordTests.Compile("unit Example; type Hidden = union Empty; end union; " + declaration + " end unit;", CompilationTarget.Library);
        Assert.Equal("FP3021", Assert.Single(result.Diagnostics).Code);
    }
}
