using System.Reflection.Metadata;

namespace Fpas.Compiler.Tests.Support;

internal static class MetadataAttributes
{
    internal static CustomAttribute Named(MetadataReader reader, IEnumerable<CustomAttributeHandle> attributes, string name)
        => attributes.Select(reader.GetCustomAttribute).Single(attribute =>
        {
            var constructor = reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
            return reader.GetString(reader.GetTypeReference((TypeReferenceHandle)constructor.Parent).Name) == name;
        });
}
