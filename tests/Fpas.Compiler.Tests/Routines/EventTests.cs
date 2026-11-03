using Fpas.Compiler.Tests.ControlFlow;
namespace Fpas.Compiler.Tests.Routines;

public sealed class EventTests
{
    [Theory]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit with Handler; unsubscribe Sys.AppDomain.CurrentDomain.ProcessExit with Handler;")]
    [InlineData("subscribe (Sys.AppDomain.CurrentDomain.ProcessExit) with Handler;")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit with procedure(Sender: object?; Args: Sys.EventArgs) begin end procedure;")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit with Handle;")]
    public void Events_accept_delegate_values_lambdas_named_routines_and_parenthesized_references(string body)
    {
        var result = Compile(body);
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
    }

    [Theory]
    [InlineData("subscribe Handler with Handler;", "FP3034")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.FriendlyName with Handler;", "FP3034")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.Missing with Handler;", "FP3034")]
    [InlineData("subscribe Sys.AppDomain.ProcessExit with Handler;", "FP3034")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit with 1;", "FP3003")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit with procedure() begin end procedure;", "FP3003")]
    [InlineData("discard Sys.AppDomain.CurrentDomain.ProcessExit;", "FP3034")]
    [InlineData("Sys.AppDomain.CurrentDomain.ProcessExit(null, Sys.EventArgs.Empty);", "FP3034")]
    [InlineData("Sys.AppDomain.CurrentDomain.ProcessExit := Handler;", "FP3034")]
    [InlineData("const Domain: Sys.AppDomain? := null; subscribe Domain.ProcessExit with Handler;", "FP3013")]
    [InlineData("subscribe Sys.AppDomain.CurrentDomain.ProcessExit Handler;", "FP2001")]
    public void Invalid_event_targets_handlers_and_ordinary_value_operations_are_rejected(string body, string code)
    {
        var result = Compile(body);
        Assert.False(result.Success); Assert.Equal(code, Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Event_accessors_invalidate_mutable_capture_nullable_facts()
    {
        var result = Compile("""
            var Text: string? := 'ready';
            const Clear: Sys.EventHandler := procedure(Sender: object?; Args: Sys.EventArgs) begin Text := null; end procedure;
            if Text <> null then
              subscribe Sys.AppDomain.CurrentDomain.ProcessExit with Clear;
              discard Text.Length;
            end if;
            """);
        Assert.False(result.Success); Assert.Equal("FP3013", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public void Event_receivers_cannot_read_unassigned_out_parameters()
    {
        var result = DiagnosticsTests.Compile("""
            program Example;
            uses System as Sys;
            procedure Wire(out Domain: Sys.AppDomain; Handler: Sys.EventHandler);
            begin subscribe Domain.ProcessExit with Handler; Domain := Sys.AppDomain.CurrentDomain; end procedure;
            begin end;
            end program;
            """);
        Assert.False(result.Success); Assert.Equal("FP3028", Assert.Single(result.Diagnostics).Code);
    }

    private static CompilationResult Compile(string body) => DiagnosticsTests.Compile($$"""
        program Example;
        uses System as Sys;
        procedure Handle(Sender: object?; Args: Sys.EventArgs); begin end procedure;
        begin
          const Handler: Sys.EventHandler := Handle;
          {{body}}
        end;
        end program;
        """);
}
