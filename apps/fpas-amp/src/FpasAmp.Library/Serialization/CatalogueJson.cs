// [Documentation](../../../doc/library.md#metadata-and-queries)

using System.Collections.Immutable;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using Fpas.Media.Tags;

namespace FpasAmp.Library.Serialization;

// Supplies generated JSON metadata for the values stored by the catalogue.
public static class CatalogueJson
{
    // Reads cached track metadata from the supplied JSON.
    public static AudioFiles.Info? ReadTrack(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Track);

    // Writes the supplied track in the existing catalogue format.
    public static string WriteTrack(AudioFiles.Info track) =>
        JsonSerializer.Serialize(track, CatalogueJsonContext.Default.Track);

    // Reads a cached cover from the supplied JSON.
    public static Pictures.Cover? ReadCover(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Cover);

    // Writes the supplied cover or a JSON null for a cached miss.
    public static string WriteCover(Pictures.Cover? cover) =>
        JsonSerializer.Serialize(cover, (JsonTypeInfo)CatalogueJsonContext.Default.Cover);

    // Reads string arrays stored in or returned by the database.
    public static string[]? ReadNames(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Names);

    // Writes the supplied strings for catalogue keys and lookups.
    public static string WriteNames(string[] names) =>
        JsonSerializer.Serialize(names, CatalogueJsonContext.Default.Names);

    // Reads saved tag overrides from JSON, including null values for removed tags.
    public static ImmutableDictionary<string, string?>? ReadTagOverrides(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.TagOverrides);

    // Writes the supplied tag overrides, including explicit removals.
    public static string WriteTagOverrides(ImmutableDictionary<string, string?> overrides) =>
        JsonSerializer.Serialize(overrides, CatalogueJsonContext.Default.TagOverrides);
}

[JsonSourceGenerationOptions(GenerationMode = JsonSourceGenerationMode.Metadata)]
[JsonSerializable(typeof(AudioFiles.Info), TypeInfoPropertyName = "Track")]
[JsonSerializable(typeof(Pictures.Cover), TypeInfoPropertyName = "Cover")]
[JsonSerializable(typeof(string[]), TypeInfoPropertyName = "Names")]
[JsonSerializable(typeof(ImmutableDictionary<string, string?>), TypeInfoPropertyName = "TagOverrides")]
internal partial class CatalogueJsonContext : JsonSerializerContext
{
}
