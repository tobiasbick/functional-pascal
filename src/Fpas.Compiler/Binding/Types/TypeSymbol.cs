using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Binding.Declarations;
namespace Fpas.Compiler.Binding.Types;

internal enum TypeKind { Unsupported, Void, String, Int32, Boolean, Double, Char, Object, Named, Array, Null, TypeParameter, MethodParameter, NativeInt, Generated, Record, Union, Parameter, ByRef, Async,
    SByte, Byte, Int16, UInt16, UInt32, Int64, UInt64, NativeUInt, Single, Decimal, Scalar, Container, Interface }

internal sealed record TypeSymbol(TypeKind Kind, ImportedType? Definition = null, bool IsNullable = false, int ParameterIndex = 0, TypeList? Arguments = null, TypeSymbol? Element = null, int Rank = 0, ClosureSymbol? Closure = null, RecordSymbol? Record = null, UnionSymbol? Union = null, GenericParameterSymbol? Parameter = null, bool NullValidated = false, AsyncMachineSymbol? Machine = null, ScalarSymbol? Scalar = null, SourceUnit? Container = null, InterfaceSymbol? Interface = null)
{
    internal static readonly TypeSymbol NativeInt = new(TypeKind.NativeInt);
    internal static readonly TypeSymbol SByte = new(TypeKind.SByte), Byte = new(TypeKind.Byte), Int16 = new(TypeKind.Int16), UInt16 = new(TypeKind.UInt16),
        UInt32 = new(TypeKind.UInt32), Int64 = new(TypeKind.Int64), UInt64 = new(TypeKind.UInt64), NativeUInt = new(TypeKind.NativeUInt), Single = new(TypeKind.Single), Decimal = new(TypeKind.Decimal);
    internal static readonly TypeSymbol Unsupported = new(TypeKind.Unsupported), Void = new(TypeKind.Void),
        String = new(TypeKind.String), Int32 = new(TypeKind.Int32), Boolean = new(TypeKind.Boolean),
        Double = new(TypeKind.Double), Char = new(TypeKind.Char), Object = new(TypeKind.Object), Null = new(TypeKind.Null);
    internal bool IsParameter => Kind == TypeKind.Parameter;
    internal bool CanHaveNull => IsReference || IsParameter && !IsValue;
    internal bool NeedsNullCheck => IsNullable || IsParameter && !Parameter!.IsNotNull && !NullValidated;
    internal bool IsValue => IsParameter ? Parameter!.Primary == "struct" : NumericFacts.NumericKind(Kind) || Kind == TypeKind.Boolean || Kind == TypeKind.Scalar && Scalar!.Base != String || Kind == TypeKind.Named && Definition!.IsValueType;
    internal bool IsReference => IsParameter ? Parameter!.IsReference : Kind is TypeKind.String or TypeKind.Object or TypeKind.Array or TypeKind.Generated or TypeKind.Record or TypeKind.Union or TypeKind.Async or TypeKind.Interface || Kind is TypeKind.Named or TypeKind.Scalar && !IsValue;
    internal bool IsInterface => Interface is not null || Definition?.Definition.Attributes.HasFlag(System.Reflection.TypeAttributes.Interface) == true;
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
        if (Container is { } container) return container.Syntax.Name.Text;
        if (Interface is { } contract) return contract.Owner.Syntax.Name.Text + "." + contract.Name +
            (Arguments is { Count: > 0 } ? "<" + string.Join(",", Arguments) + ">" : "") + (IsNullable ? "?" : "");
        var name = IsParameter ? Parameter!.Name : Kind == TypeKind.Scalar ? Scalar!.Owner.Syntax.Name.Text + "." + Scalar.Name : Kind == TypeKind.ByRef ? Element + "&" : Kind == TypeKind.Union ? Union!.Owner.Syntax.Name.Text + "." + Union.Name : Kind == TypeKind.Record ? Record!.Owner.Syntax.Name.Text + "." + (Record.UnionCase is { } member ? member.Owner.Name + "." : "") + Record.Name : Kind == TypeKind.Generated ? "<Closure" + Closure!.Id + ">" : Kind == TypeKind.Array ? Element + "[" + new string(',', Rank - 1) + "]" : Definition?.FullName ?? TypeFacts.Alias(this) ?? Kind switch
        {
            TypeKind.String => "string", TypeKind.Int32 => "int", TypeKind.Boolean => "bool", TypeKind.Double => "double", TypeKind.Char => "char",
            TypeKind.Object => "object", TypeKind.Void => "void", TypeKind.Null => "null", TypeKind.TypeParameter => "!" + ParameterIndex, TypeKind.MethodParameter => "!!" + ParameterIndex, _ => "unsupported"
        };
        if (Arguments is { Count: > 0 }) name += "<" + string.Join(",", Arguments) + ">";
        return name + (IsNullable ? "?" : "");
    }
}
