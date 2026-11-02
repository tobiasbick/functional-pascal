namespace Fpas.Compiler.Tests;

public sealed class InferenceTests
{
    [Theory]
    [InlineData("const X: int := Create();", "FP3026")]
    [InlineData("discard Id(null);", "FP3026")]
    [InlineData("discard Choose(1, 'text');", "FP3026")]
    [InlineData("discard Ref(1);", "FP3018")]
    [InlineData("const X: string? := null; discard Ref(X);", "FP3018")]
    [InlineData("const X: string? := null; const Y: string := Id(X);", "FP3013")]
    [InlineData("discard Tree.Empty;", "FP3026")]
    [InlineData("discard Tree.Leaf(null);", "FP3026")]
    [InlineData("const X: Tree<int> := Tree.Leaf('text');", "FP3026")]
    [InlineData("const X: Tree<int> := Tree.Empty();", "FP3023")]
    [InlineData("const X: Tree<int> := Tree.Leaf;", "FP3023")]
    [InlineData("discard Tree.Leaf<int>(1);", "FP3026")]
    [InlineData("discard Tree<int>.Leaf<int>(1);", "FP3026")]
    [InlineData("const X: int := Id(1); const Y := Id(2);", "FP2002")]
    [InlineData("discard Id([]);", "FP3026")]
    [InlineData("const X: function(int): int := Id;", "FP3026")]
    public void Missing_conflicting_or_unsafe_inference_is_diagnosed(string body, string code)
    {
        var result = RecordTests.Compile("""
            program Example;
            type Tree<T> = union Empty; Leaf(Value: T); end union;
            function Id<T>(Value: T): T; begin return Value; end function;
            function Create<T>(): T; where T: new(); begin return new T(); end function;
            function Choose<T>(Left: T; Right: T): T; begin return Left; end function;
            function Ref<T>(Value: T): T; where T: class; begin return Value; end function;
            begin
            """ + " " + body + " end; end program;");
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Deferred_contextual_arguments_cannot_preserve_stale_captured_null_facts()
    {
        var result = RecordTests.Compile("""
            program Example;
            type Choice<A, B> = union First(Value: A); Second(Value: B); end union;
            function Mutate(Action: procedure()): int; begin Action(); return 1; end function;
            function Read<A, B>(Left: Choice<A, B>; Length: int; Right: Choice<A, B>): int; begin return Length; end function;
            begin
              var Text: string? := 'known';
              const Other: Choice<int, string> := Choice.Second('type hint');
              discard Read(Choice.First(Mutate(procedure() begin Text := null; end procedure)), Text.Length, Other);
            end;
            end program;
            """);
        Assert.Equal("FP3013", Assert.Single(result.Diagnostics).Code);
    }
}
