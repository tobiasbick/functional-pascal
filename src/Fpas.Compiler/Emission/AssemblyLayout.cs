using Fpas.Compiler.Lowering.Async;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;

namespace Fpas.Compiler.Emission;

internal sealed class AssemblyLayout
{
    internal IReadOnlyList<UnionSymbol> Unions { get; }
    internal Dictionary<UnionSymbol, TypeDefinitionHandle> UnionTypes { get; } = [];
    internal Dictionary<UnionSymbol, TypeDefinitionHandle> UnionOwners { get; } = [];
    internal Dictionary<RecordSymbol, TypeDefinitionHandle> RecordTypes { get; } = [];
    internal Dictionary<RecordSymbol, TypeDefinitionHandle> RecordOwners { get; } = [];
    internal Dictionary<RoutineSymbol, MethodDefinitionHandle> SourceMethods { get; } = [];
    internal IReadOnlyList<RecordSymbol> Records { get; }
    internal IReadOnlyList<InterfaceSymbol> Interfaces { get; }
    internal Dictionary<InterfaceSymbol, TypeDefinitionHandle> InterfaceTypes { get; } = [];
    internal Dictionary<InterfaceSymbol, TypeDefinitionHandle> InterfaceOwners { get; } = [];
    internal Dictionary<ClosureSymbol, TypeDefinitionHandle> ClosureTypes { get; } = [];
    internal Dictionary<ClosureSymbol, TypeDefinitionHandle> ClosureOwners { get; } = [];
    internal Dictionary<ClosureSymbol, MethodDefinitionHandle> ClosureConstructors { get; } = [];
    internal Dictionary<ClosureSymbol, MethodDefinitionHandle> ClosureInvocations { get; } = [];
    internal IReadOnlyList<ClosureSymbol> Closures { get; }
    internal List<AsyncMachineSymbol> Machines { get; } = [];
    internal Dictionary<AsyncMachineSymbol, AsyncPlan> AsyncPlans { get; } = [];
    internal Dictionary<BoundRoutine, AsyncMachineSymbol> AsyncRoutines { get; } = new(ReferenceEqualityComparer.Instance);
    internal Dictionary<AsyncMachineSymbol, TypeDefinitionHandle> AsyncTypes { get; } = [];
    internal Dictionary<AsyncMachineSymbol, TypeDefinitionHandle> AsyncOwners { get; } = [];
    internal Dictionary<AsyncMachineSymbol, MethodDefinitionHandle> AsyncConstructors { get; } = [];
    internal Dictionary<AsyncMachineSymbol, MethodDefinitionHandle> AsyncMoves { get; } = [];
    internal Dictionary<AsyncMachineSymbol, MethodDefinitionHandle> AsyncSets { get; } = [];
    internal Dictionary<string, MethodDefinitionHandle> EntryBridges { get; } = new(StringComparer.Ordinal);
    internal Dictionary<ScalarSymbol, TypeDefinitionHandle> ScalarTypes { get; } = [];
    internal Dictionary<ScalarSymbol, TypeDefinitionHandle> ScalarOwners { get; } = [];
    internal Dictionary<string, TypeDefinitionHandle> Containers { get; } = new(StringComparer.Ordinal);

    internal AssemblyLayout(BoundCompilation program, IReadOnlyDictionary<BoundRoutine, AsyncPlan> asyncPlans)
    {
        Closures = program.Units.SelectMany(unit => unit.Closures).OrderBy(closure => closure.Id).ToArray();
        Unions = program.Units.SelectMany(unit => unit.Unions).ToArray();
        Records = program.Units.SelectMany(unit => unit.Records).ToArray();
        Interfaces = program.Units.SelectMany(unit => unit.Interfaces).ToArray();
        var method = 1;
        foreach (var unit in program.Units)
        {
            Containers.Add(unit.Name, MetadataTokens.TypeDefinitionHandle(Containers.Count + 2));
            foreach (var routine in unit.Routines) SourceMethods.Add(routine.Symbol, MetadataTokens.MethodDefinitionHandle(method++));
            if (unit.Routines.Any(routine => routine.Symbol.IsEntryPoint && routine.Symbol.IsAsync)) EntryBridges.Add(unit.Name, MetadataTokens.MethodDefinitionHandle(method++));
            method += unit.Aliases.Count;
        }
        var type = program.Units.Count + 2;
        foreach (var union in Unions)
        {
            UnionTypes.Add(union, MetadataTokens.TypeDefinitionHandle(type++));
            foreach (var routine in union.Methods) SourceMethods.Add(routine, MetadataTokens.MethodDefinitionHandle(method++));
        }
        foreach (var record in Records)
        {
            RecordTypes.Add(record, MetadataTokens.TypeDefinitionHandle(type++));
            foreach (var routine in record.Methods) SourceMethods.Add(routine, MetadataTokens.MethodDefinitionHandle(method++));
        }
        for (var index = 0; index < program.Units.Count; index++)
        {
            var owner = MetadataTokens.TypeDefinitionHandle(index + 2);
            foreach (var union in program.Units[index].Unions) UnionOwners.Add(union, owner);
            foreach (var record in program.Units[index].Records) RecordOwners.Add(record, record.UnionCase is { } member ? UnionTypes[member.Owner] : owner);
            foreach (var contract in program.Units[index].Interfaces) InterfaceOwners.Add(contract, owner);
            foreach (var closure in program.Units[index].Closures) ClosureOwners.Add(closure, closure.RecordContext is { } context ? RecordTypes[context] : owner);
        }
        foreach (var contract in Interfaces)
        {
            InterfaceTypes.Add(contract, MetadataTokens.TypeDefinitionHandle(type++));
            foreach (var routine in contract.Methods) SourceMethods.Add(routine, MetadataTokens.MethodDefinitionHandle(method++));
        }
        foreach (var closure in Closures)
        {
            ClosureTypes.Add(closure, MetadataTokens.TypeDefinitionHandle(type++));
            ClosureConstructors.Add(closure, MetadataTokens.MethodDefinitionHandle(method++));
            ClosureInvocations.Add(closure, MetadataTokens.MethodDefinitionHandle(method));
            foreach (var body in closure.Bodies)
            {
                if (closure.NestedBodies.Count != 0) SourceMethods.Add(body.Symbol, MetadataTokens.MethodDefinitionHandle(method));
                method++;
            }
        }
        foreach (var unit in program.Units)
        {
            var owner = MetadataTokens.TypeDefinitionHandle(program.Units.ToList().IndexOf(unit) + 2);
            void Add(BoundRoutine routine, ClosureSymbol? closure, TypeDefinitionHandle parent, string parentName)
            {
                if (!routine.Symbol.IsAsync) return;
                var machine = new AsyncMachineSymbol(routine, closure, Machines.Count, parentName);
                AsyncPlans.Add(machine, asyncPlans[routine]);
                Machines.Add(machine); AsyncRoutines.Add(routine, machine); AsyncOwners.Add(machine, parent);
                AsyncTypes.Add(machine, MetadataTokens.TypeDefinitionHandle(type++));
                AsyncConstructors.Add(machine, MetadataTokens.MethodDefinitionHandle(method++));
                AsyncMoves.Add(machine, MetadataTokens.MethodDefinitionHandle(method++));
                AsyncSets.Add(machine, MetadataTokens.MethodDefinitionHandle(method++));
            }
            foreach (var routine in unit.Routines) Add(routine, null, owner, unit.Name);
            foreach (var record in unit.Records)
                foreach (var routine in record.Bodies.Values)
                    Add(routine, null, RecordTypes[record], unit.Name + "+" + record.Name + (record.GenericParameters.Count == 0 ? "" : "`" + record.GenericParameters.Count));
            foreach (var closure in unit.Closures)
            {
                var parentName = unit.Name + (closure.RecordContext is { } context ? "+" + context.Name + (context.GenericParameters.Count == 0 ? "" : "`" + context.GenericParameters.Count) : "");
                var arity = closure.GenericParameters.Count - (closure.RecordContext?.GenericParameters.Count ?? 0);
                foreach (var body in closure.Bodies)
                    Add(body, closure, ClosureTypes[closure], parentName + "+<Closure" + closure.Id + ">" + (arity == 0 ? "" : "`" + arity));
            }
        }
        foreach (var unit in program.Units)
            foreach (var scalar in unit.Scalars)
            {
                ScalarTypes.Add(scalar, MetadataTokens.TypeDefinitionHandle(type++));
                ScalarOwners.Add(scalar, MetadataTokens.TypeDefinitionHandle(program.Units.ToList().IndexOf(unit) + 2));
            }
    }
}
