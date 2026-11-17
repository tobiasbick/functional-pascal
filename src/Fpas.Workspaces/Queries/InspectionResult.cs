using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

/// <summary>Describes an exact captured source slice and its availability.</summary>
/// <param name="Availability">Availability or omission reason.</param>
/// <param name="Span">Source location, when available.</param>
/// <param name="Text">Copied text, or null when absent, unchecked or omitted.</param>
public sealed record SourceSlice(string Availability, SourceSpan? Span, string? Text);
/// <summary>Contains declaration, body and attached-documentation source views.</summary>
/// <param name="Project">Source owner, or null for metadata.</param>
/// <param name="Declaration">Declaration slice.</param>
/// <param name="Body">Body slice or location-only view.</param>
/// <param name="Documentation">Attached documentation slice.</param>
public sealed record DeclarationSource(string? Project, SourceSlice Declaration, SourceSlice Body, SourceSlice Documentation);
/// <summary>Pairs a related declaration identity with its captured source views.</summary>
/// <param name="Definition">Resolved related declaration.</param>
/// <param name="Source">Captured views for that declaration.</param>
public sealed record RelatedDeclaration(DefinitionResult Definition, DeclarationSource Source);
/// <summary>Counts inspection omissions independently of semantic coverage.</summary>
/// <param name="SourceBudget">Number of slices omitted by the shared text budget.</param>
/// <param name="RelatedLimit">Number of source declarations omitted by the related-item limit.</param>
public sealed record InspectionOmissions(int SourceBudget, int RelatedLimit);
/// <summary>Contains a declaration inspection with bounded source text and related descriptors.</summary>
/// <param name="Definition">Selected canonical declaration.</param>
/// <param name="ContainingDeclarations">Lexically containing declarations, nearest first.</param>
/// <param name="Types">Types needed to interpret the selected surface and expression.</param>
/// <param name="Source">Selected declaration's source views.</param>
/// <param name="Related">One-hop related declarations and views.</param>
/// <param name="TotalRelatedSourceCount">Total related source declarations before paging.</param>
/// <param name="ReturnedRelatedSourceCount">Related source declarations returned.</param>
/// <param name="Omissions">Text-budget and related-limit omission counts.</param>
public sealed record InspectResult(DefinitionResult Definition, IReadOnlyList<DeclarationMatch> ContainingDeclarations,
    IReadOnlyList<SemanticType> Types, DeclarationSource Source, IReadOnlyList<RelatedDeclaration> Related,
    int TotalRelatedSourceCount, int ReturnedRelatedSourceCount, InspectionOmissions Omissions);
