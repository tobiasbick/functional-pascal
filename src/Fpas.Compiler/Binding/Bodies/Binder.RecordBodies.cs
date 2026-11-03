using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.Bound;
namespace Fpas.Compiler.Binding.Bodies;

internal sealed partial class Binder
{
    private void GenerateRecordBodies(RecordSymbol record)
    {
        var self = new BoundParameter(new("self", record.Type, -1));
        BoundParameter Parameter(RoutineSymbol method, int index) => new(method.Parameters[index]);
        RoutineSymbol Method(GeneratedMethodKind kind) => record.Methods.Single(method => method.Generated == kind);
        BoundCall Call(RoutineSymbol method, BoundExpression? receiver, params BoundExpression[] arguments) => new(record.Method(method), arguments, receiver);
        static BoundReturn Return(BoundExpression value) => new(value);
        static BoundIf When(BoundExpression condition, BoundExpression result) => new([new(condition, new([new BoundReturn(result)]))], new([]));
        static BoundBinary Same(BoundExpression left, BoundExpression right) => new(left, "=", right, TypeSymbol.Boolean, false);
        void Body(RoutineSymbol method, IReadOnlyList<BoundStatement> statements, params LocalSymbol[] locals)
            => record.Bodies.Add(method, new(method, new(statements), locals));
        foreach (var field in record.Fields)
        {
            Body(field.Getter, [Return(new BoundRecordField(field, self))]);
            if (field.Copy is not null) Body(field.Copy, [Return(new BoundNew(record.Type, record.Method(record.Constructor), record.Fields.Select(other =>
                other == field ? (BoundExpression)Parameter(field.Copy!, 0) : new BoundRecordField(other, self)).ToArray()))]);
        }
        if (record.UnionCase is { } unionCase) Body(Method(GeneratedMethodKind.Tag), [Return(new BoundInteger(unionCase.Tag))]);
        var typed = Method(GeneratedMethodKind.EqualsTyped);
        var other = Parameter(typed, 0);
        var equality = new List<BoundStatement> { When(Same(self, other), new BoundBoolean(true)), When(new BoundNullTest(other, true), new BoundBoolean(false)) };
        foreach (var field in record.Fields)
        {
            var comparer = references.Construct(references.Required("System.Collections.Generic.EqualityComparer`1"), [field.Type], source, record.Syntax.Name.Position);
            var instance = new BoundProperty(references.Property(comparer, "Default", true)!, null);
            var equals = references.FindMethod(comparer, "Equals", [field.Type, field.Type], 0, source, false);
            var same = new BoundCall(equals, [new BoundRecordField(field, self), new BoundRecordField(field, other)], instance);
            equality.Add(When(new BoundUnary("not", same, TypeSymbol.Boolean, false), new BoundBoolean(false)));
        }
        equality.Add(Return(new BoundBoolean(true)));
        Body(typed, equality);
        var objectEquals = Method(GeneratedMethodKind.EqualsObject);
        Body(objectEquals, [Return(Call(typed, self, new BoundSafeCast(Parameter(objectEquals, 0), record.Type with { IsNullable = true })))]);
        var hash = Method(GeneratedMethodKind.Hash);
        var hashType = references.Symbol(references.Required("System.HashCode"));
        var accumulator = new LocalSymbol("<hash>", hashType, 0, true);
        var hashBody = new List<BoundStatement> { new BoundBinding(accumulator, new BoundNew(hashType, null, [])) };
        if (record.UnionCase is { } member)
        {
            var addTag = references.FindMethod(hashType, "Add", [TypeSymbol.Int32], 0, source, false, methodArguments: new([TypeSymbol.Int32]));
            hashBody.Add(new BoundExpressionStatement(new BoundCall(addTag, [new BoundInteger(member.Tag)], new BoundLocal(accumulator)), false));
        }
        foreach (var field in record.Fields)
        {
            var add = references.FindMethod(hashType, "Add", [field.Type], 0, source, false, methodArguments: new([field.Type]));
            hashBody.Add(new BoundExpressionStatement(new BoundCall(add, [new BoundRecordField(field, self)], new BoundLocal(accumulator)), false));
        }
        hashBody.Add(Return(new BoundCall(references.FindMethod(hashType, "ToHashCode", [], 0, source, false), [], new BoundLocal(accumulator))));
        Body(hash, hashBody, accumulator);
        var eq = Method(GeneratedMethodKind.Equality);
        var left = Parameter(eq, 0); var right = Parameter(eq, 1);
        Body(eq, [When(Same(left, right), new BoundBoolean(true)), When(new BoundNullTest(left, true), new BoundBoolean(false)), Return(Call(typed, left, right))]);
        var ne = Method(GeneratedMethodKind.Inequality);
        Body(ne, [Return(new BoundUnary("not", Call(eq, null, Parameter(ne, 0), Parameter(ne, 1)), TypeSymbol.Boolean, false))]);
    }
}
