namespace Fpas.Debugging.Native;

internal sealed record NativeStop(string Reason, long[]? Breakpoints = null, Core.BreakpointInfo[]? ChangedBreakpoints = null,
    Core.DebugModuleIssue[]? ModuleIssues = null, Core.AsyncStepInfo? AsyncStep = null);
