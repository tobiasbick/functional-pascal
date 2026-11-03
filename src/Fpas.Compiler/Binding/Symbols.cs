using System.Reflection;
using System.Reflection.Metadata;

namespace Fpas.Compiler.Binding;

internal sealed record AssemblyIdentity(string Name, Version Version, string Culture, byte[] PublicKeyToken,
    AssemblyFlags Flags);

internal sealed record ImportedType(ReferenceAssembly Assembly, TypeDefinitionHandle Handle, string Namespace, string Name)
{
    internal ImportedType? DeclaringType
    {
        get
        {
            var handle = Definition.GetDeclaringType();
            if (handle.IsNil) return null;
            var parent = Assembly.Reader.GetTypeDefinition(handle);
            return new(Assembly, handle, Assembly.Reader.GetString(parent.Namespace), Assembly.Reader.GetString(parent.Name));
        }
    }
    internal bool IsPublic => DeclaringType is { } parent
        ? (Definition.Attributes & TypeAttributes.VisibilityMask) == TypeAttributes.NestedPublic && parent.IsPublic
        : (Definition.Attributes & TypeAttributes.VisibilityMask) == TypeAttributes.Public;
    internal string FullName => DeclaringType is { } parent ? parent.FullName + "+" + Name : Namespace.Length == 0 ? Name : Namespace + "." + Name;
    internal TypeDefinition Definition => Assembly.Reader.GetTypeDefinition(Handle);
    internal bool IsValueType => FullName != "System.Enum" && (BaseName is "System.ValueType" or "System.Enum");
    internal bool IsEnum => BaseName == "System.Enum";
    private string BaseName
    {
        get
        {
            var handle = Definition.BaseType;
            if (handle.IsNil) return "";
            if (handle.Kind == HandleKind.TypeReference)
            {
                var type = Assembly.Reader.GetTypeReference((TypeReferenceHandle)handle);
                return Assembly.Reader.GetString(type.Namespace) + "." + Assembly.Reader.GetString(type.Name);
            }
            if (handle.Kind == HandleKind.TypeDefinition)
            {
                var type = Assembly.Reader.GetTypeDefinition((TypeDefinitionHandle)handle);
                return Assembly.Reader.GetString(type.Namespace) + "." + Assembly.Reader.GetString(type.Name);
            }
            return "";
        }
    }
}
internal sealed record ImportedMethod(TypeSymbol Owner, string Name, TypeSymbol Result, IReadOnlyList<TypeSymbol> Parameters,
    bool IsStatic, bool IsVirtual = false, TypeSymbol? SignatureResult = null, IReadOnlyList<TypeSymbol>? SignatureParameters = null, bool ReturnsByRef = false, TypeList? MethodArguments = null, MethodDefinitionHandle DefinitionHandle = default, RoutineSymbol? SourceMethod = null, IReadOnlyList<ParameterSymbol>? ParameterInfo = null)
{
    internal ParameterSymbol Parameter(int index) => ParameterInfo?[index] ?? new("", Parameters[index], index);
    internal bool Flexible => ParameterInfo?.Any(parameter => parameter.Mode != "" || parameter.Default is not null) == true;
    internal IReadOnlyList<TypeSymbol> DelegateParameters => Parameters.Select((type, index) => Parameter(index).ByRef ? TypeSymbol.ByRef(type) : type).ToArray();
}
internal sealed record ImportedField(TypeSymbol Owner, string Name, TypeSymbol Type, bool IsStatic, bool IsReadOnly, object? Constant = null, bool IsLiteral = false, TypeSymbol? SignatureType = null);
internal sealed record ImportedProperty(TypeSymbol Type, ImportedMethod? Getter, ImportedMethod? Setter);
internal sealed record NamespaceImport(string Namespace) : ImportTarget;
internal abstract record ImportTarget;
internal sealed record SourceImport(SourceUnit Unit) : ImportTarget;
internal sealed record ClrImport(ImportedType Type) : ImportTarget;
internal sealed class LocalSymbol(string name, TypeSymbol type, int slot, bool mutable = false)
{
    internal string Name { get; } = name;
    internal TypeSymbol Type { get; } = type;
    internal int Slot { get; } = slot;
    internal bool Mutable { get; } = mutable;
    internal bool Captured { get; set; }
    internal Syntax.DocumentationComment? Documentation { get; set; }
}
internal sealed record ParameterSymbol(string Name, TypeSymbol Type, int Index, string Mode = "")
{
    internal bool ByRef => Mode is "ref" or "out" or "in";
    internal TypeSymbol SignatureType => ByRef ? TypeSymbol.ByRef(Type) : Type;
    internal BoundExpression? Default { get; set; }
}
internal sealed record BoundAddress(BoundExpression Target, string Mode) : BoundExpression(TypeSymbol.ByRef(Target.Type));
internal sealed record BoundDereference(BoundExpression Address) : BoundExpression(Address.Type.Element!);
internal sealed record RoutineSymbol(string Name, TypeSymbol Result, IReadOnlyList<ParameterSymbol> Parameters,
    bool IsPublic, bool IsEntryPoint, int Index, bool IsInstance = false, RecordSymbol? Owner = null, GeneratedMethodKind Generated = GeneratedMethodKind.None, UnionSymbol? UnionOwner = null, IReadOnlyList<GenericParameterSymbol>? TypeParameters = null, Syntax.RoutineSyntax? Declaration = null, bool IsAsync = false)
{
    internal IReadOnlyList<GenericParameterSymbol> GenericParameters => TypeParameters ?? [];
    internal TypeSymbol BodyResult => !IsAsync ? Result : Result.TypeArguments.Count == 1 ? Result.TypeArguments[0] : TypeSymbol.Void;
}
internal sealed record BoundRoutine(RoutineSymbol Symbol, BoundBlock Body, IReadOnlyList<LocalSymbol> Locals,
    IReadOnlyList<BoundContract>? Requires = null, IReadOnlyList<BoundContract>? Ensures = null, LocalSymbol? Result = null, Syntax.SourceText? Source = null, IReadOnlyList<BoundAttribute>? Attributes = null);
internal sealed record BoundContract(BoundExpression Condition, string Message, SourceLocation? Location = null);
internal sealed record BoundUnit(string Name, IReadOnlyList<BoundRoutine> Routines, IReadOnlyList<ClosureSymbol> Closures, IReadOnlyList<RecordSymbol> Records, IReadOnlyList<UnionSymbol> Unions, IReadOnlyList<ScalarSymbol> Scalars, IReadOnlyList<AliasSymbol> Aliases, IReadOnlyList<GlobalSymbol> Globals);
internal sealed record BoundCompilation(AssemblyIdentity Runtime, bool IsExecutable, IReadOnlyList<BoundUnit> Units);

internal abstract record BoundStatement;
internal sealed record BoundSequencePoint(SourceLocation Location, BoundStatement Statement) : BoundStatement;
internal sealed record BoundBinding(LocalSymbol Local, BoundExpression Initializer) : BoundStatement;
internal sealed record BoundAssignment(BoundExpression Target, BoundExpression Value) : BoundStatement;
internal sealed record BoundThrow(BoundExpression? Exception) : BoundStatement;
internal sealed record BoundUsing(LocalSymbol Local, BoundExpression Initializer, ImportedMethod Dispose, BoundBlock Body, bool ConditionalDispose = false, bool IsAsync = false) : BoundStatement;
internal sealed record BoundDispose(BoundUsing Resource) : BoundStatement;
internal sealed record BoundCatch(TypeSymbol ExceptionType, LocalSymbol? Local, BoundExpression? Filter, BoundBlock Body);
internal sealed record BoundTry(BoundBlock Body, IReadOnlyList<BoundCatch> Catches, BoundBlock? Finally) : BoundStatement;
internal sealed record BoundExpressionStatement(BoundExpression Expression, bool Discard) : BoundStatement;
internal sealed record BoundReturn(BoundExpression? Expression) : BoundStatement;
internal sealed record BoundBlock(IReadOnlyList<BoundStatement> Statements, IReadOnlyList<LocalSymbol>? ScopeLocals = null) : BoundStatement;
internal sealed record BoundIfArm(BoundExpression Condition, BoundBlock Body);
internal sealed record BoundIf(IReadOnlyList<BoundIfArm> Arms, BoundBlock Else) : BoundStatement;
internal sealed record BoundWhile(BoundExpression Condition, BoundBlock Body) : BoundStatement;
internal sealed record BoundRepeat(BoundExpression Condition, BoundBlock Body, SourceLocation ConditionLocation) : BoundStatement;
internal sealed record BoundFor(LocalSymbol Counter, LocalSymbol Limit, BoundExpression Lower, BoundExpression Upper,
    BoundExpression Test, BoundExpression AtLimit, BoundExpression Advance, BoundBlock Body) : BoundStatement;
internal sealed record BoundLoopExit(bool IsBreak) : BoundStatement;
internal abstract record BoundExpression(TypeSymbol Type);
internal sealed record BoundString(string Value) : BoundExpression(TypeSymbol.String);
internal sealed record BoundInteger(int Value) : BoundExpression(TypeSymbol.Int32);
internal sealed record BoundBoolean(bool Value) : BoundExpression(TypeSymbol.Boolean);
internal sealed record BoundDouble(double Value) : BoundExpression(TypeSymbol.Double);
internal sealed record BoundLocal(LocalSymbol Local, TypeSymbol? FlowType = null) : BoundExpression(FlowType ?? Local.Type);
internal sealed record BoundParameter(ParameterSymbol Parameter, TypeSymbol? FlowType = null) : BoundExpression(FlowType ?? Parameter.Type);
internal sealed record BoundCall(ImportedMethod Method, IReadOnlyList<BoundExpression> Arguments, BoundExpression? Receiver = null, IReadOnlyList<ParameterSymbol>? WrittenParameters = null) : BoundExpression(Method.Result);
internal sealed record BoundNew(TypeSymbol Result, ImportedMethod? Constructor, IReadOnlyList<BoundExpression> Arguments, IReadOnlyList<ParameterSymbol>? WrittenParameters = null) : BoundExpression(Result);
internal sealed record BoundNull() : BoundExpression(TypeSymbol.Null);
internal sealed record BoundTypeOf(TypeSymbol Operand, ImportedMethod Resolve) : BoundExpression(Resolve.Result.NonNullable);
internal sealed record BoundField(ImportedField Field, BoundExpression? Receiver) : BoundExpression(Field.Type);
internal sealed record BoundProperty(ImportedProperty Property, BoundExpression? Receiver, IReadOnlyList<BoundExpression>? Arguments = null) : BoundExpression(Property.Type);
internal sealed record BoundCoalesce(BoundExpression Left, BoundExpression Right, TypeSymbol Result) : BoundExpression(Result);
internal sealed record BoundNullTest(BoundExpression Operand, bool EqualsNull) : BoundExpression(TypeSymbol.Boolean);
internal sealed record BoundUnary(string Operator, BoundExpression Operand, TypeSymbol Result, bool Checked) : BoundExpression(Result);
internal sealed record BoundBinary(BoundExpression Left, string Operator, BoundExpression Right, TypeSymbol Result, bool Checked) : BoundExpression(Result);
internal sealed record BoundConversion(BoundExpression Operand, TypeSymbol Target, bool Checked) : BoundExpression(Target);
internal sealed record BoundConditionalArm(BoundExpression Condition, BoundExpression Value);
internal sealed record BoundConditional(IReadOnlyList<BoundConditionalArm> Arms, BoundExpression Else, TypeSymbol Result) : BoundExpression(Result);

internal sealed record BoundCharacter(char Value) : BoundExpression(TypeSymbol.Char);
internal sealed record BoundNewArray(TypeSymbol Result, IReadOnlyList<BoundExpression> Lengths) : BoundExpression(Result);
internal sealed record BoundArrayLiteral(TypeSymbol Result, IReadOnlyList<BoundExpression> Elements) : BoundExpression(Result);
internal sealed record BoundArrayIndex(BoundExpression Receiver, IReadOnlyList<BoundExpression> Indices) : BoundExpression(Receiver.Type.Element!);
internal sealed record BoundArrayLength(BoundExpression Receiver) : BoundExpression(TypeSymbol.Int32);
internal sealed record ClrTypeImport(TypeSymbol Type) : ImportTarget;

internal sealed record RecordImport(RecordSymbol Record) : ImportTarget;
internal sealed record BoundRecordField(RecordFieldSymbol Field, BoundExpression Receiver) : BoundExpression(Field.Type);
internal sealed record BoundSequence(IReadOnlyList<BoundBinding> Bindings, BoundExpression Value) : BoundExpression(Value.Type);
internal sealed record BoundSafeCast(BoundExpression Value, TypeSymbol Target) : BoundExpression(Target);
