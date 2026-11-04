using Fpas.Compiler.Analysis;
using static Fpas.Compiler.Tests.Analysis.Support.SemanticAnalysis;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class SourceOperationTests
{
    [Fact]
    public void Bodies_own_only_their_operations_including_nested_lambdas_and_initializers()
    {
        const string source = """
            program Ownership;
            const Initial: int := Seed();
            type Sample = record Value: int := Seed(); end record;
            function Seed(): int; begin return 1; end function;
            function Outer(): int;
            begin
              function Inner(): int; begin return Seed(); end function;
              const Callback: function(): int := function(): int => Seed();
              return Inner() + Callback();
            end function;
            begin discard Outer(); end;
            end program;
            """;
        var snapshot = Analyze(source);
        var outer = Body(snapshot, "Outer"); var inner = Body(snapshot, "Inner");
        Assert.Equal(outer.Id, inner.Parent);
        var lambda = Assert.Single(snapshot.Bodies, body => body.Kind == "lambda");
        Assert.Null(lambda.Symbol); Assert.Equal(outer.Id, lambda.Parent);
        Assert.Equal(["callable_reference", "delegate_invoke", "invoke"], Owned(outer).Select(site => site.Kind).Order(StringComparer.Ordinal));
        Assert.Equal("Inner", snapshot.Symbol(Assert.Single(Owned(outer), site => site.Kind == "invoke").Target!).Name);
        foreach (var body in new[] { inner, lambda, Assert.Single(snapshot.Bodies, body => body.Kind == "global_initializer"), Assert.Single(snapshot.Bodies, body => body.Kind == "field_default") })
            Assert.Equal("Seed", snapshot.Symbol(Assert.Single(Owned(body)).Target!).Name);
        Assert.Empty(Owned(Body(snapshot, "Seed")));
        Assert.Equal("Outer", snapshot.Symbol(Assert.Single(Owned(Assert.Single(snapshot.Bodies, body => body.Kind == "program_entry"))).Target!).Name);
        Assert.All(snapshot.Operations, site => Assert.Equal("body", site.Phase));
        return;
        IEnumerable<SourceOperation> Owned(SourceBody body) => snapshot.Operations.Where(site => site.Body == body.Id);
    }

    [Fact]
    public void Explicit_accessors_construction_events_and_delegate_calls_keep_exact_operations()
    {
        var snapshot = Analyze("""
            program Operations;
            uses System.Text as Text, System.Collections.Generic as C, System.Timers as Timers;
            begin
              const Buffer: Text.StringBuilder := new Text.StringBuilder();
              Buffer.Length := 0;
              discard Buffer.Length;
              const Values: C.List<int> := new C.List<int>();
              Values.Add(2);
              Values[0] := Values[0] + 1;
              using const Timer: Timers.Timer := new Timers.Timer(100.0);
              const Handler: Timers.ElapsedEventHandler := procedure(Sender: object?; Event: Timers.ElapsedEventArgs) begin end procedure;
              subscribe Timer.Elapsed with Handler;
              unsubscribe Timer.Elapsed with Handler;
            end;
            end program;
            """);
        var entry = Assert.Single(snapshot.Bodies, body => body.Kind == "program_entry");
        var sites = snapshot.Operations.Where(site => site.Body == entry.Id).ToArray();
        Assert.Equal(3, sites.Count(site => site.Kind == "construct"));
        Assert.Equal(["get_Item", "get_Length"], sites.Where(site => site.Kind == "property_get").Select(site => site.Operation!.Name).Order(StringComparer.Ordinal));
        Assert.Equal(["set_Item", "set_Length"], sites.Where(site => site.Kind == "property_set").Select(site => site.Operation!.Name).Order(StringComparer.Ordinal));
        Assert.Equal("add_Elapsed", Assert.Single(sites, site => site.Kind == "event_add").Operation!.Name);
        Assert.Equal("remove_Elapsed", Assert.Single(sites, site => site.Kind == "event_remove").Operation!.Name);
        Assert.Equal("using_protocol", Assert.Single(snapshot.ExcludedOperations).Kind);
        Assert.DoesNotContain(sites, site => site.Operation?.Name == "Dispose");
        Assert.All(sites.Where(site => site.Kind != "callable_reference"), site => Assert.NotNull(site.Operation));
    }

    [Fact]
    public void Static_contracts_and_callable_references_do_not_guess_runtime_targets()
    {
        var snapshot = Analyze("""
            program Dispatch;
            uses System as S, System.Numerics as N;
            function Read(Value: int): int; begin return Value; end function;
            function Read(Value: string): int; begin return Value.Length; end function;
            function Apply(Value: function(int): int): int; begin return Value(1); end function;
            function Compare(Value: S.IComparable<int>): int; begin return Value.CompareTo(1); end function;
            function Add<T>(Left: T; Right: T): T;
            where T: N.IAdditionOperators<T, T, T>;
            begin return Left + Right; end function;
            begin discard Apply(Read); end;
            end program;
            """);
        var reference = Assert.Single(snapshot.Operations, site => site.Kind == "callable_reference");
        Assert.Equal("int", snapshot.Types.Single(type => type.Id == reference.Signature!.Parameters[0].Type).Display);
        Assert.Equal("Read", snapshot.Symbol(reference.Target!).Name);
        var invocation = Assert.Single(snapshot.Operations, site => site.Kind == "delegate_invoke");
        Assert.Null(invocation.Target); Assert.Equal("runtime_delegate_target", invocation.UnknownReason);
        Assert.Equal("Value", snapshot.Symbol(invocation.Receiver!.Symbol!).Name);
        var contract = Assert.Single(snapshot.Operations, site => site.Dispatch == "interface");
        Assert.Equal("CompareTo", contract.Operation!.Name); Assert.Equal("runtime_dispatch_target", contract.UnknownReason);
        var constrained = Assert.Single(snapshot.Operations, site => site.Kind == "operator");
        Assert.Equal("constrained", constrained.Dispatch); Assert.Equal("op_CheckedAddition", constrained.Operation!.Name);
    }

    [Fact]
    public void Async_and_enumeration_protocols_are_counted_without_fabricated_runtime_calls()
    {
        var snapshot = Analyze("""
            program Protocols;
            uses System.Threading.Tasks as Tasks, System.IO as Io;
            async function Work(): Tasks.Task<int>;
            begin
              await using const Stream: Io.MemoryStream := new Io.MemoryStream();
              await Tasks.Task.Delay(1);
              var Total: int := 0;
              for Value: int in [1, 2] do Total := Total + Value; end for;
              return Total;
            end function;
            begin end;
            end program;
            """);
        Assert.Equal(["async_using_protocol", "await_protocol", "foreach_protocol"], snapshot.ExcludedOperations.Select(site => site.Kind).Order(StringComparer.Ordinal));
        Assert.Equal([".ctor", "Delay"], snapshot.Operations.Select(site => site.Operation!.Name).Order(StringComparer.Ordinal));
        Assert.All(snapshot.Operations, site => Assert.Equal(Body(snapshot, "Work").Id, site.Body));
    }

    [Fact]
    public void Contract_getters_belong_to_the_routine_with_separate_phases()
    {
        var snapshot = Analyze("""
            program Contracts;
            function Size(Value: string): int;
            requires Value.Length > 0;
            ensures (Result: int): Result = Value.Length;
            begin return Value.Length; end function;
            begin end;
            end program;
            """);
        Assert.Equal(["body", "ensures", "requires"], snapshot.Operations.Select(site => site.Phase).Order(StringComparer.Ordinal));
        Assert.All(snapshot.Operations, site => { Assert.Equal("property_get", site.Kind); Assert.Equal(Body(snapshot, "Size").Id, site.Body); });
        Assert.Equal(3, Body(snapshot, "Size").Regions.Count);
    }

    private static SourceBody Body(SemanticSnapshot snapshot, string name)
        => Assert.Single(snapshot.Bodies, body => body.Symbol is { } symbol && snapshot.Symbol(symbol).Name == name);
}
