using Fpas.Compiler.Analysis;
using Fpas.Compiler.Tests.Analysis.Support;

namespace Fpas.Compiler.Tests.Analysis;

public sealed class SemanticConstructTests
{
    [Fact]
    public void Records_variants_generics_aliases_and_named_labels_resolve_to_their_declarations()
    {
        const string source = """
            program Data;
            type Number = int;
            type Point = record public X: Number; end record;
            type Choice = variant None; Some(Value: int); end variant;
            function Identity<T>(Value: T): T; begin return Value; end function;
            begin
              const P: Point := new Point(X := Identity<int>(21));
              const Q: Point := P with X := 42; end with;
              const C: Choice := Choice.Some(Value := Q.X);
              case C of
                when Choice.None: begin end;
                when Choice.Some(Value := const Item: int): begin discard Item; end;
              end case;
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var field = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "X" && symbol.Declaration is not null);
        foreach (var marker in new[] { "X := Identity", "X := 42", "X);" })
            Assert.Equal(field.Id, Select(snapshot, source, marker).Id);
        var alias = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "Number");
        Assert.Equal(alias.Id, Select(snapshot, source, "Number; end record").Id);
        var some = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "variant_case" && symbol.Name == "Some");
        Assert.Equal(some.Id, Select(snapshot, source, "Some(Value := Q").Id);
        Assert.Equal(some.Id, Select(snapshot, source, "Some(Value := const").Id);
        var item = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "Item");
        Assert.Equal(item.Id, Select(snapshot, source, "Item; end").Id);
        Assert.Equal("int", TypeAt(snapshot, source, "Identity<int>(21)").Callable is { } callable ? snapshot.Types.Single(type => type.Id == callable.Result).Display : null);
        AssertConsistent(snapshot);
    }

    [Fact]
    public void Modes_contracts_array_types_and_namespace_selections_keep_their_source_meaning()
    {
        const string source = """
            program Modes;
            uses System as S;
            procedure Work(ref Changed: int; out Output: int; in Input: int);
            begin Changed := Changed + Input; Output := Changed; end procedure;
            function Positive(Value: int): int;
            requires Value > 0;
            ensures (Returned: int): Returned > 0;
            begin return Value; end function;
            begin
              var Value: int := 21;
              var Output: int := 0;
              Work(ref Value, out Output, in Value);
              const Values: array[,] of int := new int[1, 2];
              Values[0, 0] := Positive(Output);
              S.Console.WriteLine(Values.Length);
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var reference = snapshot.OccurrenceAt("input.fpas", source.IndexOf("ref Value", StringComparison.Ordinal) + 4)!;
        Assert.Equal("read_write", reference.Role);
        Assert.Equal("write", snapshot.OccurrenceAt("input.fpas", source.LastIndexOf("out Output", StringComparison.Ordinal) + 4)!.Role);
        var returned = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "Returned");
        Assert.Equal(returned.Id, Select(snapshot, source, "Returned >").Id);
        Assert.Equal(2, TypeAt(snapshot, source, "Values.Length").Rank);
        var ns = source.IndexOf("S.Console", StringComparison.Ordinal);
        Assert.Equal("alias", snapshot.Symbol(snapshot.OccurrenceAt("input.fpas", ns)!.Symbol).Kind);
        Assert.Null(snapshot.TypeAt("input.fpas", ns));
        AssertConsistent(snapshot);
    }

    [Fact]
    public void Events_attributes_async_and_cleanup_have_explicit_facts_without_generated_occurrences()
    {
        const string source = """
            program Infrastructure;
            uses System as S, System.Timers as Timers, System.Threading.Tasks as Tasks;
            [S.ObsoleteAttribute('Example', DiagnosticId := 'ExampleCode')]
            procedure Handler(Sender: object?; Args: Timers.ElapsedEventArgs);
            begin end procedure;
            async function Later<T>(Input: T): Tasks.Task<T>;
            begin return await Tasks.Task.FromResult<T>(Input); end function;
            begin
              using const Timer: Timers.Timer := new Timers.Timer();
              subscribe Timer.Elapsed with Handler;
              unsubscribe Timer.Elapsed with Handler;
              discard Later<int>(42);
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var elapsed = Select(snapshot, source, "Elapsed with Handler");
        Assert.Equal("event", elapsed.Kind); Assert.Equal("Elapsed", elapsed.Metadata!.Name);
        Assert.Equal(2, snapshot.Occurrences.Count(occurrence => occurrence.Symbol == elapsed.Id));
        var named = Select(snapshot, source, "DiagnosticId :=");
        Assert.Equal("property", named.Kind); Assert.Equal("System.ObsoleteAttribute", named.Metadata!.Type);
        var parameter = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "parameter" && symbol.Name == "Input");
        Assert.Equal(parameter.Id, Select(snapshot, source, "Input); end function").Id);
        Assert.DoesNotContain(snapshot.Symbols, symbol => symbol.Name.Contains("Awaiter", StringComparison.Ordinal) || symbol.Name.Contains("StateMachine", StringComparison.Ordinal));
        AssertConsistent(snapshot);
    }

    [Fact]
    public void Inferred_variant_owners_delegate_properties_and_managed_storage_keep_exact_checked_types()
    {
        const string source = """
            program ExactTypes;
            type Choice<T> = variant None; Some(Value: T); end variant;
            type Holder = record public Callback: function(int): int; end record;
            procedure Keep(in Value: string?); begin end procedure;
            begin
              const Selected: Choice<int> := Choice.Some(42);
              const H: Holder := new Holder(function(Value: int): int => Value + 1);
              discard H.Callback(42);
              var Text: string? := 'value';
              Keep(in Text);
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var owner = TypeAt(snapshot, source, "Choice.Some");
        Assert.Equal("variant", owner.Kind); Assert.Equal("int", snapshot.Types.Single(type => type.Id == Assert.Single(owner.Arguments)).Display);
        var callback = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "property" && symbol.Name == "Callback");
        Assert.Equal(callback.Id, Select(snapshot, source, "Callback(42)").Id);
        Assert.True(TypeAt(snapshot, source, "Text);").Nullable);
        AssertConsistent(snapshot);
    }

    [Fact]
    public void Generic_parameters_and_enum_initializer_uses_retain_original_declarations()
    {
        const string source = """
            program Original;
            type Flags = enum First = 1; Second = First + 1; Third = cast<int>(Flags.First) + Second; end enum;
            function Identity<T>(Value: T): T; begin return Value; end function;
            begin const Flag: Flags := Flags.Second; discard Identity<int>(42); end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var generic = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "type_parameter");
        var parameterType = snapshot.Types.Single(type => type.Id == generic.Type);
        Assert.Equal("method", parameterType.GenericOwner); Assert.Equal(0, parameterType.GenericIndex);
        var first = Assert.Single(snapshot.Symbols, symbol => symbol.Name == "First");
        Assert.Equal(first.Id, Select(snapshot, source, "First + 1").Id);
        Assert.Equal(first.Id, Select(snapshot, source, "First) + Second").Id);
        Assert.Equal("enum", snapshot.Types.Single(type => type.Id == first.Type).Kind);
        AssertConsistent(snapshot);
    }

    [Fact]
    public void Interface_calls_select_the_static_contract_and_generic_receivers_select_their_type_parameter()
    {
        const string source = """
            program StaticTargets;
            uses System.Numerics as N;
            type IReader = protocol function Read(): int; end protocol;
            type Reader = record implements IReader
              public function Read(): int; begin return 42; end function;
            end record;
            function Zero<T>(): T; where T: N.INumber<T>;
            begin return T.Zero; end function;
            begin
              const Contract: IReader := new Reader();
              discard Contract.Read(); discard Zero<int>();
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var offset = source.IndexOf("Contract.Read", StringComparison.Ordinal) + "Contract.".Length;
        var method = snapshot.Symbol(snapshot.OccurrenceAt("input.fpas", offset)!.Symbol);
        Assert.Equal(3, method.Declaration!.Line);
        var generic = Assert.Single(snapshot.Symbols, symbol => symbol.Kind == "type_parameter");
        Assert.Equal(generic.Id, Select(snapshot, source, "T.Zero").Id);
        Assert.Equal("parameter", TypeAt(snapshot, source, "T.Zero").Kind);
        AssertConsistent(snapshot);
    }

    [Fact]
    public void External_generic_definitions_are_shared_while_each_use_keeps_its_checked_instantiation()
    {
        const string source = """
            program Definitions;
            uses System.Linq as L;
            begin
              discard L.Enumerable.Repeat<int>(42, 1);
              discard L.Enumerable.Repeat<string>('text', 1);
            end;
            end program;
            """;
        var snapshot = SemanticAnalysis.Analyze(source);
        var first = Select(snapshot, source, "Repeat<int>"); var second = Select(snapshot, source, "Repeat<string>");
        Assert.Equal(first.Id, second.Id); Assert.Null(first.Type); Assert.Null(first.Callable);
        Assert.Equal(1, first.Metadata!.Arity);
        Assert.Equal(new[] { "!!0", "System.Int32" }, first.Metadata.Parameters);
        foreach (var (marker, expected) in new[] { ("Repeat<int>", "int"), ("Repeat<string>", "string") })
        {
            var callable = TypeAt(snapshot, source, marker).Callable!;
            var result = snapshot.Types.Single(type => type.Id == callable.Result);
            Assert.Equal(expected, snapshot.Types.Single(type => type.Id == Assert.Single(result.Arguments)).Display);
        }
        AssertConsistent(snapshot);
    }

    private static SemanticSymbol Select(SemanticSnapshot snapshot, string source, string marker)
    {
        var offset = source.IndexOf(marker, StringComparison.Ordinal); Assert.True(offset >= 0);
        return snapshot.Symbol(Assert.IsType<SymbolOccurrence>(snapshot.OccurrenceAt("input.fpas", offset)).Symbol);
    }

    private static SemanticType TypeAt(SemanticSnapshot snapshot, string source, string marker)
        => snapshot.Types.Single(type => type.Id == snapshot.TypeAt("input.fpas", source.IndexOf(marker, StringComparison.Ordinal))!.Type);

    private static void AssertConsistent(SemanticSnapshot snapshot)
    {
        foreach (var occurrence in snapshot.Occurrences)
        {
            Assert.NotNull(snapshot.Symbol(occurrence.Symbol)); Assert.True(occurrence.Span.Length > 0);
            Assert.InRange(occurrence.Span.Offset + occurrence.Span.Length, 0, snapshot.Documents.Single(document => document.Path == occurrence.Span.Path).Text.Length);
        }
        foreach (var expression in snapshot.Expressions) Assert.Contains(snapshot.Types, type => type.Id == expression.Type);
    }
}
