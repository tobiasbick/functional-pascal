using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Queries;

public sealed record SourceSlice(string Availability, SourceSpan? Span, string? Text);
public sealed record DeclarationSource(string? Project, SourceSlice Declaration, SourceSlice Body, SourceSlice Documentation);
public sealed record RelatedDeclaration(DefinitionResult Definition, DeclarationSource Source);
public sealed record InspectionOmissions(int SourceBudget, int RelatedLimit);
public sealed record InspectResult(DefinitionResult Definition, IReadOnlyList<DeclarationMatch> ContainingDeclarations,
    IReadOnlyList<SemanticType> Types, DeclarationSource Source, IReadOnlyList<RelatedDeclaration> Related,
    int TotalRelatedSourceCount, int ReturnedRelatedSourceCount, InspectionOmissions Omissions);
