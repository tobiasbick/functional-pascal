// [Documentation](../../../docs/architecture/workspaces.md#input-consistency-and-ownership)
using Fpas.Compiler.Analysis;

namespace Fpas.Workspaces.Analysis;

/// <summary>Describes which semantic regions are available in one compilation.</summary>
/// <param name="Project">Owning project path.</param>
/// <param name="Availability">Complete, partial or unavailable analysis state.</param>
/// <param name="Regions">Checked and unavailable source regions.</param>
public sealed record ProjectSemanticCoverage(string Project, string Availability, IReadOnlyList<SemanticRegion> Regions);
/// <summary>Describes semantic coverage independently of query pagination.</summary>
/// <param name="Complete">Whether all selected project inputs and semantic regions are available.</param>
/// <param name="Projects">Per-project coverage.</param>
public sealed record WorkspaceSemanticCoverage(bool Complete, IReadOnlyList<ProjectSemanticCoverage> Projects);
