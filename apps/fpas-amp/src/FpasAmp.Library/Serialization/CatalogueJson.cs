// [Documentation](../../../doc/library.md#metadata-and-queries)

using System.Collections.Immutable;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using Fpas.Media.Tags;

namespace FpasAmp.Library.Serialization;

/// <summary>Serializes catalogue values with generated JSON metadata.</summary>
public static class CatalogueJson
{
    /// <summary>Reads cached track metadata.</summary>
    /// <param name="json">The stored JSON representation.</param>
    /// <returns>The track metadata, or null for a JSON null.</returns>
    public static AudioFiles.Info? ReadTrack(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Track);

    /// <summary>Writes track metadata for the catalogue.</summary>
    /// <param name="track">The track metadata to store.</param>
    /// <returns>The track's serialized JSON.</returns>
    public static string WriteTrack(AudioFiles.Info track) =>
        JsonSerializer.Serialize(track, CatalogueJsonContext.Default.Track);

    /// <summary>Reads a cached cover, including a cached miss.</summary>
    /// <param name="json">The stored JSON representation.</param>
    /// <returns>The cover, or null for a cached miss.</returns>
    public static Pictures.Cover? ReadCover(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Cover);

    /// <summary>Writes a cached cover, including a cached miss.</summary>
    /// <param name="cover">The cover to store, or null for a cached miss.</param>
    /// <returns>The cover's serialized JSON or a JSON null.</returns>
    public static string WriteCover(Pictures.Cover? cover) =>
        JsonSerializer.Serialize(cover, (JsonTypeInfo)CatalogueJsonContext.Default.Cover);

    /// <summary>Reads a string array returned by the database.</summary>
    /// <param name="json">The stored or queried JSON representation.</param>
    /// <returns>The strings in order, or null for a JSON null.</returns>
    public static string[]? ReadNames(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.Names);

    /// <summary>Writes ordered strings for catalogue keys and lookups.</summary>
    /// <param name="names">The strings to store.</param>
    /// <returns>The serialized JSON array.</returns>
    public static string WriteNames(string[] names) =>
        JsonSerializer.Serialize(names, CatalogueJsonContext.Default.Names);

    /// <summary>Reads saved tag overrides and explicit tag removals.</summary>
    /// <param name="json">The stored JSON representation.</param>
    /// <returns>The overrides, with null values marking removed tags, or null for a JSON null.</returns>
    public static ImmutableDictionary<string, string?>? ReadTagOverrides(string json) =>
        JsonSerializer.Deserialize(json, CatalogueJsonContext.Default.TagOverrides);

    /// <summary>Writes tag overrides and explicit tag removals.</summary>
    /// <param name="overrides">The overrides, with null values marking removed tags.</param>
    /// <returns>The overrides' serialized JSON.</returns>
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
