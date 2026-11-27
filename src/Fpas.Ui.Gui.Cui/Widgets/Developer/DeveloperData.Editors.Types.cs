// [Documentation](../../../../docs/ui/gui/developer-tools/README.md)
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui.Widgets.Developer;

/// <summary>Editable key and value pair.</summary>
/// <param name="Key">Key supplied to the widget.</param>
/// <param name="Value">Value supplied to the widget.</param>
/// <param name="Enabled">Enabled supplied to the widget.</param>
/// <param name="Secret">Secret supplied to the widget.</param>
public sealed record CuiDevPair(string Key, string Value, bool Enabled, bool Secret);

/// <summary>Allowed field in a native query.</summary>
/// <param name="Name">Name supplied to the widget.</param>
/// <param name="Kind">Kind supplied to the widget.</param>
/// <param name="Choices">Choices supplied to the widget.</param>
public sealed record CuiDevField(string Name, int Kind, string[] Choices);

/// <summary>One group or condition in a query.</summary>
/// <param name="Depth">Depth supplied to the widget.</param>
/// <param name="Group">Group supplied to the widget.</param>
/// <param name="Any">Any supplied to the widget.</param>
/// <param name="Field">Field supplied to the widget.</param>
/// <param name="Operator">Operator supplied to the widget.</param>
/// <param name="Value">Value supplied to the widget.</param>
public sealed record CuiDevRule(int Depth, bool Group, bool Any, int Field, int Operator, string Value);

/// <summary>Feature flag state across environments.</summary>
/// <param name="Key">Key supplied to the widget.</param>
/// <param name="Description">Description supplied to the widget.</param>
/// <param name="On">On supplied to the widget.</param>
/// <param name="Rollout">Rollout supplied to the widget.</param>
public sealed record CuiDevFlag(string Key, string? Description, bool[] On, int[] Rollout);

/// <summary>Schedule expression and native validation.</summary>
/// <param name="Expression">Expression supplied to the widget.</param>
/// <param name="Valid">Valid supplied to the widget.</param>
/// <param name="Next">Next supplied to the widget.</param>
public sealed record CuiScheduleProposal(string Expression, bool Valid, long Next);

/// <summary>Current regex text and match summary.</summary>
/// <param name="Pattern">Pattern supplied to the widget.</param>
/// <param name="Sample">Sample supplied to the widget.</param>
/// <param name="Modifiers">Modifiers supplied to the widget.</param>
/// <param name="Valid">Valid supplied to the widget.</param>
/// <param name="Matches">Matches supplied to the widget.</param>
/// <param name="Error">CUI's diagnostic for invalid input or a match that exceeds its budget.</param>
/// <param name="ModifiersChanged">Whether input changed the match flags in this frame.</param>
public sealed record CuiRegexProposal(string Pattern, string Sample, int Modifiers, bool Valid, int Matches, string? Error = null, bool ModifiersChanged = false);

/// <summary>Current HTTP draft and actions reported by the widget.</summary>
/// <param name="Method">Selected HTTP method index.</param>
/// <param name="Url">Current URL draft.</param>
/// <param name="Send">Whether the user requests submission.</param>
/// <param name="Cancel">Whether the user requests cancellation.</param>
/// <param name="Changed">Whether the user edited the method or URL in this frame.</param>
public sealed record CuiRequestProposal(int Method, string Url, bool Send, bool Cancel, bool Changed);

/// <summary>Changed feature flag cell.</summary>
/// <param name="Index">Index supplied to the widget.</param>
/// <param name="Environment">Environment supplied to the widget.</param>
/// <param name="On">On supplied to the widget.</param>
/// <param name="Rollout">Rollout supplied to the widget.</param>
public sealed record CuiFlagProposal(int Index, int Environment, bool On, int Rollout);
