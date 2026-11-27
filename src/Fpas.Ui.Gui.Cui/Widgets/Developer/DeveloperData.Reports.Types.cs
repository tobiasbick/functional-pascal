// [Documentation](../../../../docs/ui/gui/more-developer-tools/README.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Depth">Depth supplied to CUI.</param>
/// <param name="Status">Status supplied to CUI.</param>
/// <param name="Duration">Duration supplied to CUI.</param>
/// <param name="Message">Message supplied to CUI.</param>
public sealed record CuiDevTestItem(string Name, int Depth, int Status, float Duration, string? Message);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="FunctionName">Function Name supplied to CUI.</param>
/// <param name="File">File supplied to CUI.</param>
/// <param name="Line">Line supplied to CUI.</param>
/// <param name="Column">Column supplied to CUI.</param>
/// <param name="External">External supplied to CUI.</param>
/// <param name="Source">Source supplied to CUI.</param>
/// <param name="SourceFirst">Source First supplied to CUI.</param>
public sealed record CuiDevStackFrame(string? FunctionName, string? File, int Line, int Column, bool External, string[] Source, int SourceFirst);

/// <summary>Describes one service's availability history.</summary>
/// <param name="Name">Name shown above the daily bars.</param>
/// <param name="Days">Daily availability from oldest to newest. Negative values represent missing data.</param>
public sealed record CuiDevService(string Name, float[] Days);

/// <summary>Describes one series of event counts in a time histogram.</summary>
/// <param name="Name">Name shown in the legend and hover labels.</param>
/// <param name="Counts">One count per bucket, or null for all zeros.</param>
/// <param name="Tone">Native tone from zero through eight. Zero uses the series palette.</param>
public sealed record CuiDevSeries(string Name, float[]? Counts, int Tone);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Path">Path supplied to CUI.</param>
/// <param name="Lines">Lines supplied to CUI.</param>
/// <param name="LinesHit">Lines Hit supplied to CUI.</param>
/// <param name="Branches">Branches supplied to CUI.</param>
/// <param name="BranchesHit">Branches Hit supplied to CUI.</param>
/// <param name="Functions">Functions supplied to CUI.</param>
/// <param name="FunctionsHit">Functions Hit supplied to CUI.</param>
public sealed record CuiDevCoverageFile(string Path, int Lines, int LinesHit, int Branches, int BranchesHit, int Functions, int FunctionsHit);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Hash">Hash supplied to CUI.</param>
/// <param name="Author">Author supplied to CUI.</param>
/// <param name="Summary">Summary supplied to CUI.</param>
/// <param name="Time">Time supplied to CUI.</param>
public sealed record CuiDevRevision(string Hash, string Author, string? Summary, long Time);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Location">Location supplied to CUI.</param>
/// <param name="Type">Type supplied to CUI.</param>
/// <param name="Required">Required supplied to CUI.</param>
/// <param name="Description">Description supplied to CUI.</param>
public sealed record CuiDevParameter(string Name, string Location, string Type, bool Required, string? Description);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Method">Method supplied to CUI.</param>
/// <param name="Path">Path supplied to CUI.</param>
/// <param name="Summary">Summary supplied to CUI.</param>
/// <param name="Tag">Tag supplied to CUI.</param>
/// <param name="Deprecated">Deprecated supplied to CUI.</param>
/// <param name="Parameters">Parameters supplied to CUI.</param>
public sealed record CuiDevEndpoint(string Method, string Path, string? Summary, string? Tag, bool Deprecated, CuiDevParameter[] Parameters);
