using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private void CheckRecordMember(TypeSymbol owner, string name, bool writing, int position)
    {
        if (references.RecordInfo(owner) is not { } info) return;
        var field = info.Fields.FirstOrDefault(field => field.Name == name);
        if (field is not null)
        {
            if (writing) throw source.Error("FP3009", "Record fields are immutable; use with to create an updated value.", position);
            if (!field.IsPublic && (owner.Record is null || owner.Record != recordContext))
                throw source.Error("FP3011", $"Record field '{name}' is private.", position);
        }
        if (owner.Record is { } record && record.UserMethods.TryGetValue(name, out var methods) && methods.All(method => !method.IsPublic) && record != recordContext)
            throw source.Error("FP3011", $"Record method '{name}' is private.", position);
    }

    private BoundExpression BindRecordNew(NewSyntax syntax, RecordInfo info, bool check)
    {
        if (syntax.Type.Nullable) throw source.Error("FP3005", "Cannot instantiate a nullable type.", syntax.Position);
        if (!info.CanConstruct && (info.Type.Record is null || info.Type.Record != recordContext))
            throw source.Error("FP3011", "The record constructor is private; use a public factory.", syntax.Position);
        var (bindings, values) = BindDataArguments(syntax.Arguments, info.Fields.Select(field => (field.Name, field.Type, field.Default)).ToArray(), check, syntax.Position);
        foreach (var argument in syntax.Arguments.OfType<NamedArgumentSyntax>())
            catalog.Analysis?.NamedProperty(source, argument.Name, info.Fields.Single(field => field.Name == argument.Name.Text).Getter, "argument");
        ForgetCapturedFacts();
        return new BoundSequence(bindings, new BoundNew(info.Type, info.Constructor, values));
    }

    private BoundExpression BindWith(WithSyntax syntax, bool check)
    {
        var receiver = Receiver(syntax.Receiver, check);
        var info = references.RecordInfo(receiver.Type) ?? throw source.Error("FP3022", "with requires an FPAS record.", syntax.Position);
        var bindings = new List<BoundBinding>();
        BoundExpression result = Save(receiver, bindings);
        var names = new HashSet<string>(StringComparer.Ordinal);
        var updates = new List<(RecordFieldInfo Field, BoundExpression Value)>();
        foreach (var update in syntax.Updates)
        {
            var field = info.Fields.FirstOrDefault(field => field.Name == update.Name.Text);
            if (field is null || !names.Add(update.Name.Text)) throw source.Error("FP3022", "Unknown or duplicate record update.", update.Name.Position);
            if (!field.IsPublic && (info.Type.Record is null || info.Type.Record != recordContext))
                throw source.Error("FP3011", $"Record field '{field.Name}' is private.", update.Name.Position);
            catalog.Analysis?.NamedProperty(source, update.Name, field.Getter, "write");
            var value = Convert(BindExpression(update.Value, check, field.Type), field.Type, update.Value.Position);
            updates.Add((field, Save(value, bindings)));
        }
        foreach (var update in updates) result = new BoundCall(update.Field.Copy, [update.Value], result);
        ForgetCapturedFacts();
        return new BoundSequence(bindings, result);
    }

    private void BindRecord(RecordSymbol record)
    {
        Binder Context()
        {
            var binder = new Binder(unit, catalog, references, recordContext: record, unitClosures: closures);
            foreach (var import in imports) binder.imports.Add(import.Key, import.Value);
            return binder;
        }
        foreach (var syntax in record.Syntax.Methods)
        {
            var symbol = record.UserMethods[syntax.Name.Text].Single(symbol => ReferenceEquals(symbol.Declaration, syntax));
            record.Bodies.Add(symbol, Context().BindRoutine(symbol, syntax.Statements, syntax.Name.Position, syntax.Contracts));
        }
        foreach (var field in record.Fields.Where(field => field.Default is not null))
        {
            using var sourceBody = catalog.Analysis?.Initializer(source, field, field.Syntax.Default!, "field_default");
            var binder = Context();
            binder.currentRoutine = field.Default!;
            binder.scopes.Push(new(StringComparer.Ordinal));
            var value = binder.Convert(binder.BindExpression(field.Syntax.Default!, expected: field.Type), field.Type, field.Syntax.Name.Position);
            binder.FinishNestedBindings();
            record.Bodies.Add(field.Default!, new(field.Default!, new([new BoundReturn(value)]), binder.locals.ToArray()));
        }
        GenerateRecordBodies(record);
    }
}
