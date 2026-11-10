using Fpas.Compiler.Binding.Types;
using Fpas.Compiler.Binding.Symbols;
using Fpas.Compiler.Binding.References;

namespace Fpas.Compiler.Binding.Declarations;

internal sealed record NamespaceImport(string Namespace) : ImportTarget;
internal abstract record ImportTarget;
internal sealed record SourceImport(SourceUnit Unit) : ImportTarget;
internal sealed record ClrImport(ImportedType Type) : ImportTarget;
internal sealed record ClrTypeImport(TypeSymbol Type) : ImportTarget;

internal sealed record RecordImport(RecordSymbol Record) : ImportTarget;
internal sealed record VariantImport(VariantSymbol Variant) : ImportTarget;
internal sealed record InterfaceImport(InterfaceSymbol Interface) : ImportTarget;
