namespace Fpas.Compiler.Binding;

internal enum TypeKind { Unsupported, Void, String, Int32, Boolean, Double, Char, Object, Named, Array, Null, TypeParameter, MethodParameter, NativeInt, Generated, Record, Union, Parameter, ByRef, Async }

internal sealed record TypeSymbol(TypeKind Kind, ImportedType? Definition = null, bool IsNullable = false, int ParameterIndex = 0, TypeList? Arguments = null, TypeSymbol? Element = null, int Rank = 0, ClosureSymbol? Closure = null, RecordSymbol? Record = null, UnionSymbol? Union = null, GenericParameterSymbol? Parameter = null, bool NullValidated = false, AsyncMachineSymbol? Machine = null)
{
    internal static readonly TypeSymbol NativeInt = new(TypeKind.NativeInt);
    internal static readonly TypeSymbol Unsupported = new(TypeKind.Unsupported), Void = new(TypeKind.Void),
        String = new(TypeKind.String), Int32 = new(TypeKind.Int32), Boolean = new(TypeKind.Boolean),
        Double = new(TypeKind.Double), Char = new(TypeKind.Char), Object = new(TypeKind.Object), Null = new(TypeKind.Null);
    internal bool IsParameter => Kind == TypeKind.Parameter;
    internal bool CanHaveNull => IsReference || IsParameter && !IsValue;
    internal bool NeedsNullCheck => IsNullable || IsParameter && !Parameter!.IsNotNull && !NullValidated;
    internal bool IsValue => IsParameter ? Parameter!.Primary == "struct" : Kind is TypeKind.Int32 or TypeKind.Boolean or TypeKind.Double or TypeKind.Char || Kind == TypeKind.Named && Definition!.IsValueType;
    internal bool IsReference => IsParameter ? Parameter!.IsReference : Kind is TypeKind.String or TypeKind.Object or TypeKind.Array or TypeKind.Generated or TypeKind.Record or TypeKind.Union or TypeKind.Async || Kind == TypeKind.Named && !IsValue;
    internal bool NullableValue => IsNullable && IsValue;
    internal TypeSymbol NonNullable => this with { IsNullable = false, NullValidated = IsParameter };
    internal TypeList TypeArguments => NullableValue ? new([NonNullable]) : Arguments ?? TypeList.Empty;
    internal static TypeSymbol Array(TypeSymbol element, int rank = 1) => new(TypeKind.Array, Element: element, Rank: rank);
    internal static TypeSymbol ByRef(TypeSymbol element) => new(TypeKind.ByRef, Element: element);
    internal TypeSymbol Erased => this with
    {
        IsNullable = IsValue && IsNullable,
        NullValidated = false,
        Element = Element?.Erased,
        Arguments = Arguments is null ? null : new(Arguments.Select(argument => argument.Erased))
    };
    public override string ToString()
    {
        var name = IsParameter ? Parameter!.Name : Kind == TypeKind.ByRef ? Element + "&" : Kind == TypeKind.Union ? Union!.Owner.Syntax.Name.Text + "." + Union.Name : Kind == TypeKind.Record ? Record!.Owner.Syntax.Name.Text + "." + (Record.UnionCase is { } member ? member.Owner.Name + "." : "") + Record.Name : Kind == TypeKind.Generated ? "<Closure" + Closure!.Id + ">" : Kind == TypeKind.Array ? Element + "[" + new string(',', Rank - 1) + "]" : Definition?.FullName ?? Kind switch
        {
            TypeKind.String => "string", TypeKind.Int32 => "int", TypeKind.Boolean => "bool", TypeKind.Double => "double", TypeKind.Char => "char",
            TypeKind.Object => "object", TypeKind.Void => "void", TypeKind.Null => "null", TypeKind.TypeParameter => "!" + ParameterIndex, TypeKind.MethodParameter => "!!" + ParameterIndex, _ => "unsupported"
        };
        if (Arguments is { Count: > 0 }) name += "<" + string.Join(",", Arguments) + ">";
        return name + (IsNullable ? "?" : "");
    }
}
