using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Days">Days supplied to CUI.</param>
public sealed record CuiDevService(string Name, float[] Days);

/// <summary>Describes an input row for a native developer widget.</summary>
/// <param name="Name">Name supplied to CUI.</param>
/// <param name="Counts">Counts supplied to CUI.</param>
/// <param name="Tone">Tone supplied to CUI.</param>
public sealed record CuiDevSeries(string Name, float[] Counts, int Tone);

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

public sealed partial class CuiWindow
{
    private static unsafe NativeDevTestItem* PackTestItem(ChartMemory memory, CuiDevTestItem[] items)
    {
        var native = new NativeDevTestItem[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Depth = item.Depth,
                Status = item.Status,
                Duration = item.Duration,
                Message = memory.Text(item.Message),
            };
        }
        return memory.Copy<NativeDevTestItem>(native);
    }

    private static unsafe NativeDevStackFrame* PackStackFrame(ChartMemory memory, CuiDevStackFrame[] items)
    {
        var native = new NativeDevStackFrame[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                FunctionName = memory.Text(item.FunctionName),
                File = memory.Text(item.File),
                Line = item.Line,
                Column = item.Column,
                External = item.External ? (byte)1 : (byte)0,
                Source = memory.Labels(item.Source),
                SourceCount = item.Source.Length,
                SourceFirst = item.SourceFirst,
            };
        }
        return memory.Copy<NativeDevStackFrame>(native);
    }

    private static unsafe NativeDevService* PackService(ChartMemory memory, CuiDevService[] items)
    {
        var native = new NativeDevService[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Days = memory.Copy<float>(item.Days),
                DaysCount = item.Days.Length,
            };
        }
        return memory.Copy<NativeDevService>(native);
    }

    private static unsafe NativeDevSeries* PackSeries(ChartMemory memory, CuiDevSeries[] items)
    {
        var native = new NativeDevSeries[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Counts = memory.Copy<float>(item.Counts),
                Tone = item.Tone,
            };
        }
        return memory.Copy<NativeDevSeries>(native);
    }

    private static unsafe NativeDevCoverageFile* PackCoverageFile(ChartMemory memory, CuiDevCoverageFile[] items)
    {
        var native = new NativeDevCoverageFile[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Path = memory.Text(item.Path),
                Lines = item.Lines,
                LinesHit = item.LinesHit,
                Branches = item.Branches,
                BranchesHit = item.BranchesHit,
                Functions = item.Functions,
                FunctionsHit = item.FunctionsHit,
            };
        }
        return memory.Copy<NativeDevCoverageFile>(native);
    }

    private static unsafe NativeDevRevision* PackRevision(ChartMemory memory, CuiDevRevision[] items)
    {
        var native = new NativeDevRevision[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Hash = memory.Text(item.Hash),
                Author = memory.Text(item.Author),
                Summary = memory.Text(item.Summary),
                Time = item.Time,
            };
        }
        return memory.Copy<NativeDevRevision>(native);
    }

    private static unsafe NativeDevParameter* PackParameter(ChartMemory memory, CuiDevParameter[] items)
    {
        var native = new NativeDevParameter[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Location = memory.Text(item.Location),
                Type = memory.Text(item.Type),
                Required = item.Required ? (byte)1 : (byte)0,
                Description = memory.Text(item.Description),
            };
        }
        return memory.Copy<NativeDevParameter>(native);
    }

    private static unsafe NativeDevEndpoint* PackEndpoint(ChartMemory memory, CuiDevEndpoint[] items)
    {
        var native = new NativeDevEndpoint[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Method = memory.Text(item.Method),
                Path = memory.Text(item.Path),
                Summary = memory.Text(item.Summary),
                Tag = memory.Text(item.Tag),
                Deprecated = item.Deprecated ? (byte)1 : (byte)0,
                Parameters = PackParameter(memory, item.Parameters),
                ParametersCount = item.Parameters.Length,
            };
        }
        return memory.Copy<NativeDevEndpoint>(native);
    }
}
