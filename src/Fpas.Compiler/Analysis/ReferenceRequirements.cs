using Fpas.Compiler.Binding.Bound;
using Fpas.Compiler.Binding.References;
using Fpas.Compiler.Syntax;

namespace Fpas.Compiler.Analysis;

internal static class ReferenceRequirements
{
    internal static void Validate(BoundCompilation program, ReferenceCatalog references)
    {
        // These metadata-only dependencies must not first become errors while writing a PE.
        references.Required("System.Diagnostics.DebuggableAttribute");
        references.Required("System.Diagnostics.DebuggableAttribute+DebuggingModes");
        foreach (var unit in program.Units)
        {
            foreach (var record in unit.Records)
            {
                record.Owner.Source.CheckCancellation();
                if (record.VariantCase is not null) references.Required("Fpas.Core.VariantCaseAttribute");
                if (record.VariantCase is null && record.Fields.Count != 0) references.Required("Fpas.Core.RecordFieldAttribute");
            }
            if (unit.Scalars.Any(scalar => scalar.Syntax.Category != ScalarCategory.Enum))
                references.Required("System.Collections.Generic.EqualityComparer`1");
        }
    }
}
