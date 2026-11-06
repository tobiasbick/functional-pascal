using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Analysis;

public sealed record ProjectSemanticCoverage(string Project, string Availability, IReadOnlyList<SemanticRegion> Regions);
public sealed record WorkspaceSemanticCoverage(bool Complete, IReadOnlyList<ProjectSemanticCoverage> Projects);
