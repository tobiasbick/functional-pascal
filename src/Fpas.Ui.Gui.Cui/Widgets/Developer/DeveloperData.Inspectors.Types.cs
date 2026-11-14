using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Severity">Severity supplied to CUI.</param>
/// <param name="Message">Message supplied to CUI.</param>
/// <param name="File">File supplied to CUI.</param>
/// <param name="Line">Line supplied to CUI.</param>
/// <param name="Column">Column supplied to CUI.</param>
/// <param name="Source">Source supplied to CUI.</param>
/// <param name="Code">Code supplied to CUI.</param>
public sealed record CuiDevDiagnostic(int Severity, string Message, string? File, int Line, int Column, string? Source, string? Code);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Stage">Stage supplied to CUI.</param>
/// <param name="Status">Status supplied to CUI.</param>
/// <param name="Duration">Duration supplied to CUI.</param>
/// <param name="AllowFailure">Allow Failure supplied to CUI.</param>
public sealed record CuiDevJob(string Name, int Stage, int Status, float Duration, bool AllowFailure);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Modifiers">Modifiers supplied to CUI.</param>
/// <param name="Key">Key supplied to CUI.</param>
/// <param name="Action">Action supplied to CUI.</param>
/// <param name="Category">Category supplied to CUI.</param>
public sealed record CuiDevBinding(int Modifiers, int Key, string Action, int Category);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Hash">Hash supplied to CUI.</param>
/// <param name="Message">Message supplied to CUI.</param>
/// <param name="Author">Author supplied to CUI.</param>
/// <param name="Date">Date supplied to CUI.</param>
/// <param name="Refs">Refs supplied to CUI.</param>
/// <param name="Parent">Parent supplied to CUI.</param>
/// <param name="MergeParent">Merge Parent supplied to CUI.</param>
public sealed record CuiDevCommit(string Hash, string Message, string? Author, string? Date, string? Refs, int Parent, int MergeParent);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Label">Label supplied to CUI.</param>
/// <param name="Detail">Detail supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
public sealed record CuiDevGraphNode(string Label, string? Detail, int Tone);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="From">From supplied to CUI.</param>
/// <param name="To">To supplied to CUI.</param>
public sealed record CuiDevEdge(int From, int To);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Method">Method supplied to CUI.</param>
/// <param name="Status">Status supplied to CUI.</param>
/// <param name="Type">Type supplied to CUI.</param>
/// <param name="Size">Size supplied to CUI.</param>
/// <param name="Start">Start supplied to CUI.</param>
/// <param name="Queued">Queued supplied to CUI.</param>
/// <param name="Dns">Dns supplied to CUI.</param>
/// <param name="Connect">Connect supplied to CUI.</param>
/// <param name="Tls">Tls supplied to CUI.</param>
/// <param name="Wait">Wait supplied to CUI.</param>
/// <param name="Download">Download supplied to CUI.</param>
public sealed record CuiDevRequest(string Name, string Method, int Status, string? Type, double Size, double Start, float Queued, float Dns, float Connect, float Tls, float Wait, float Download);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Collapsed">Collapsed supplied to CUI.</param>
public sealed record CuiDevTrack(string Name, bool Collapsed);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Start">Start supplied to CUI.</param>
/// <param name="End">End supplied to CUI.</param>
/// <param name="Track">Track supplied to CUI.</param>
/// <param name="Depth">Depth supplied to CUI.</param>
/// <param name="ColorKey">Color Key supplied to CUI.</param>
public sealed record CuiDevSpan(string Name, double Start, double End, int Track, int Depth, uint ColorKey);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Time">Time supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
public sealed record CuiDevMarker(string Name, double Time, int Tone);
