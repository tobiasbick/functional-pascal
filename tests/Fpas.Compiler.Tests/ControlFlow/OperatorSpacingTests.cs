using Fpas.Compiler.Tests.Support;

namespace Fpas.Compiler.Tests.ControlFlow;

public sealed class OperatorSpacingTests
{
    [Theory]
    [InlineData(">=")]
    [InlineData("> =")]
    [InlineData(">\t=")]
    [InlineData(">\n=")]
    [InlineData(">// gap\n=")]
    public void Greater_equal_comparisons_require_adjacent_operator_characters(string operation)
    {
        var source = "program Comparison;begin const X:int:=2;if X " + operation + " 1 then return;end if;end;end program;";
        var result = TestCompilation.Compile(source);
        if (operation == ">=") Assert.True(result.Success, string.Join('\n', result.Diagnostics));
        else
        {
            Assert.False(result.Success);
            Assert.Contains(result.Diagnostics, diagnostic => diagnostic.Code == "FP2003" && diagnostic.Location?.Offset == source.IndexOf('=', source.IndexOf("if X", StringComparison.Ordinal)));
        }
    }
}
