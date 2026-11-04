using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Flow;
using Fpas.Compiler.Binding.Declarations;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder(SourceUnit unit, SourceCatalog catalog, ReferenceCatalog references, Binder? parent = null, ClosureSymbol? closure = null, RecordSymbol? recordContext = null, List<ClosureSymbol>? unitClosures = null)
{
    private readonly SourceText source = unit.Source;
    private readonly Dictionary<string, ImportTarget> imports = new(StringComparer.Ordinal);
    private readonly Dictionary<string, ParameterSymbol> parameters = new(StringComparer.Ordinal);
    private readonly IReadOnlyDictionary<string, List<RoutineSymbol>> routines = unit.Routines;
    private readonly Stack<Dictionary<string, LocalSymbol>> scopes = new();
    private readonly List<LocalSymbol> locals = [];
    private readonly List<ClosureSymbol> closures = parent?.closures ?? unitClosures ?? [];
    private readonly Dictionary<object, CaptureSymbol> captures = new(ReferenceEqualityComparer.Instance);
    private RoutineSymbol currentRoutine = null!;
    private int loopDepth;
    private bool arithmeticChecked = true;
    private readonly HashSet<object> nonNull = [];
    private readonly Stack<(int LoopDepth, int CatchDepth)> finallyBoundaries = new();
    private int catchDepth;

    internal BoundUnit Bind()
    {
        source.CheckCancellation();
        var program = unit.Syntax;
        InitializeImports();
        var methods = new List<BoundRoutine>();
        if (unit.Initializer is { } initializer) methods.Add(BindRoutine(initializer, [], program.Name.Position));
        foreach (var routine in program.Routines)
            methods.Add(BindRoutine(routines[routine.Name.Text].Single(symbol => ReferenceEquals(symbol.Declaration, routine)), routine.Statements, routine.Name.Position, routine.Contracts));
        if (unit.EntryPoint is { } entry)
            methods.Add(BindRoutine(entry, program.Statements, program.Name.Position));
        foreach (var union in unit.Unions.Values) BindUnion(union);
        foreach (var record in unit.RecordTypes) BindRecord(record);
        BindTypeAttributes();
        return new(program.Name.Text, methods, closures.ToArray(), unit.RecordTypes.ToArray(), unit.Unions.Values.ToArray(), unit.Scalars.Values.ToArray(), unit.Aliases.Values.Where(alias => alias.Syntax.IsPublic).ToArray(), unit.Globals.Values.ToArray(), unit.Interfaces.Values.ToArray());
    }

    private void InitializeImports()
    {
        var targets = new HashSet<string>(StringComparer.Ordinal);
        foreach (var import in unit.Syntax.Imports)
        {
            if (imports.ContainsKey(import.Alias.Text) || !targets.Add(import.Target.Text) || import.Alias.Text == "args")
                throw source.Error("FP3004", "Import aliases must be unique; a type may be imported only once and 'args' is reserved.", import.Alias.Position);
            var target = catalog.Resolve(import.Target.Text, import.Target.Position, source, references);
            imports.Add(import.Alias.Text, target);
            catalog.Analysis?.Import(unit, import, target);
        }
    }

    private BoundRoutine BindRoutine(RoutineSymbol symbol, IReadOnlyList<StatementSyntax> body, int position, IReadOnlyList<ContractSyntax>? contracts = null)
    {
        source.CheckCancellation();
        currentRoutine = symbol;
        catalog.Analysis?.Routine(symbol, source);
        if (symbol.IsAsync && (symbol.Result.IsNullable || symbol.Result.Definition?.FullName is not ("System.Threading.Tasks.Task" or "System.Threading.Tasks.Task`1") || symbol.Parameters.Any(parameter => parameter.ByRef)))
            throw source.Error("FP3030", "Async functions return non-null Task or Task<T> and cannot have ref/out/in parameters.", position);
        locals.Clear();
        scopes.Clear();
        parameters.Clear();
        nonNull.Clear();
        operatorCalls.Clear();
        potentialNestedCaptures = NestedCaptureNames(body).ToHashSet(StringComparer.Ordinal);
        if (symbol.Owner is not null && symbol.IsInstance) parameters.Add("self", new("self", symbol.Owner.Type, -1));
        foreach (var parameter in symbol.Parameters)
            parameters.Add(parameter.Name, parameter);
        var attributes = BindAttributes(symbol.Declaration?.Attributes, System.AttributeTargets.Method);
        if (symbol.IsAsync && attributes.Any(attribute => attribute.Constructor.Owner.Definition?.FullName == "System.Runtime.CompilerServices.AsyncStateMachineAttribute"))
            throw source.Error("FP3035", "The compiler owns an async routine's state-machine attribute.", position);
        var requires = BindContracts(contracts, ensures: false, out _);
        var block = ReferenceEquals(symbol, unit.Initializer) ? BindGlobalInitializers() : BindBlock(body);
        if (symbol.BodyResult != TypeSymbol.Void && FlowAnalysis.Of(block).HasFlag(ExitFlow.Next))
            throw source.Error("FP3007", $"Function '{symbol.Name}' must return a value on every normal path.", position);
        nonNull.Clear();
        var ensures = BindContracts(contracts, ensures: true, out var result);
        var routine = new BoundRoutine(symbol, block, locals.ToArray(), requires, ensures, result, source, attributes);
        if (parent is null) FinishNestedBindings();
        new ParameterFlow(source, position).Validate(routine);
        return routine;
    }

    private LocalSymbol Allocate(string name, TypeSymbol type, bool mutable = false)
    {
        var local = new LocalSymbol(name, type, locals.Count, mutable);
        locals.Add(local);
        return local;
    }

    private void Declare(LocalSymbol local, int position)
    {
        if (scopes.Peek().ContainsKey(local.Name) || ScopeRoutines(scopes.Peek())?.Methods.ContainsKey(local.Name) == true || imports.ContainsKey(local.Name) ||
            scopes.Count == 1 && parameters.ContainsKey(local.Name) || currentRoutine.IsEntryPoint && local.Name == "args" || recordContext is not null && local.Name == "self")
            throw source.Error("FP3004", $"Name '{local.Name}' is already bound or reserved.", position);
        scopes.Peek().Add(local.Name, local);
        catalog.Analysis?.Local(local, source, position);
    }

    private BoundLocal Save(BoundExpression value, List<BoundBinding> bindings)
    {
        var local = Allocate("<value-temp>", value.Type);
        bindings.Add(new(local, value));
        return new(local);
    }

    private LocalSymbol? FindLocal(string name)
    {
        foreach (var scope in scopes)
        {
            if (scope.TryGetValue(name, out var local))
                return local;
            if (ScopeRoutines(scope)?.Methods.ContainsKey(name) == true) return null;
        }
        return null;
    }

    private IReadOnlyDictionary<string, GenericParameterSymbol> TypeParameters =>
        (closure?.GenericParameters ?? recordContext?.GenericParameters ?? currentRoutine.InterfaceOwner?.GenericParameters ?? []).Concat(currentRoutine.GenericParameters).ToDictionary(parameter => parameter.Name, StringComparer.Ordinal);

    private TypeSymbol BindType(TypeSyntax type) => catalog.ResolveType(type, unit, references, TypeParameters);

    private BoundExpression Convert(BoundExpression value, TypeSymbol target, int position, bool explicitCast = false, bool check = true)
    {
        if (value is BoundArrayLiteral array && LiteralVector(target) is not null && AcceptsArrayLiteral(array, target))
            value = RetargetArrayLiteral(array, target, position);
        if (value is BoundDelegate callable && Retarget(callable, target) is { } convertedDelegate) return convertedDelegate;
        if (value.Type == target && target != TypeSymbol.Void)
            return value;
        if (ScalarConversion(value, target, position, explicitCast, check) is { } scalar) return scalar;
        if (value.Type.CanHaveNull && value.Type.NeedsNullCheck && target.IsReference && !target.IsNullable && !(value.Type.IsParameter && target.IsParameter && value.Type.Parameter == target.Parameter))
            throw source.Error("FP3013", "A nullable reference needs validation before conversion to a non-null reference.", position);
        if (target != TypeSymbol.Void && (references.CanAssign(value.Type, target) || !explicitCast && ConstantFits(value, target) || explicitCast &&
            (TypeFacts.Numeric(value.Type.NonNullable) && TypeFacts.Numeric(target.NonNullable) ||
             value.Type.IsReference && target.IsReference || value.Type == TypeSymbol.Object && (target.IsValue || target.IsParameter) ||
             target.IsParameter && value.Type.IsReference || value.Type.IsParameter && target.IsParameter ||
             value.Type.NullableValue && value.Type.NonNullable == target)))
            return new BoundConversion(value, target, check);
        var code = value.Type.IsNullable && !target.IsNullable || value.Type == TypeSymbol.Null ? "FP3013" : "FP3002";
        throw source.Error(code, $"Cannot convert {TypeFacts.Display(value.Type)} to {TypeFacts.Display(target)}.", position);
    }

    private BoundExpression Condition(ExpressionSyntax syntax, bool check = true)
        => Convert(BindExpression(syntax, check), TypeSymbol.Boolean, syntax.Position);
}
