using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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

public sealed partial class CuiWindow
{
    private static unsafe NativeDevDiagnostic* PackDiagnostic(ChartMemory memory, CuiDevDiagnostic[] items)
    {
        var native = new NativeDevDiagnostic[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Severity = item.Severity,
                Message = memory.Text(item.Message),
                File = memory.Text(item.File),
                Line = item.Line,
                Column = item.Column,
                Source = memory.Text(item.Source),
                Code = memory.Text(item.Code),
            };
        }
        return memory.Copy<NativeDevDiagnostic>(native);
    }

    private static unsafe NativeDevJob* PackJob(ChartMemory memory, CuiDevJob[] items)
    {
        var native = new NativeDevJob[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Stage = item.Stage,
                Status = item.Status,
                Duration = item.Duration,
                AllowFailure = item.AllowFailure ? (byte)1 : (byte)0,
            };
        }
        return memory.Copy<NativeDevJob>(native);
    }

    private static unsafe NativeDevBinding* PackBinding(ChartMemory memory, CuiDevBinding[] items)
    {
        var native = new NativeDevBinding[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Modifiers = item.Modifiers,
                Key = item.Key,
                Action = memory.Text(item.Action),
                Category = item.Category,
            };
        }
        return memory.Copy<NativeDevBinding>(native);
    }

    private static unsafe NativeDevCommit* PackCommit(ChartMemory memory, CuiDevCommit[] items)
    {
        var native = new NativeDevCommit[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Hash = memory.Text(item.Hash),
                Message = memory.Text(item.Message),
                Author = memory.Text(item.Author),
                Date = memory.Text(item.Date),
                Refs = memory.Text(item.Refs),
                Parent = item.Parent,
                MergeParent = item.MergeParent,
            };
        }
        return memory.Copy<NativeDevCommit>(native);
    }

    private static unsafe NativeDevGraphNode* PackGraphNode(ChartMemory memory, CuiDevGraphNode[] items)
    {
        var native = new NativeDevGraphNode[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Label = memory.Text(item.Label),
                Detail = memory.Text(item.Detail),
                Tone = item.Tone,
            };
        }
        return memory.Copy<NativeDevGraphNode>(native);
    }

    private static unsafe NativeDevEdge* PackEdge(ChartMemory memory, CuiDevEdge[] items)
    {
        var native = new NativeDevEdge[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                From = item.From,
                To = item.To,
            };
        }
        return memory.Copy<NativeDevEdge>(native);
    }

    private static unsafe NativeDevRequest* PackRequest(ChartMemory memory, CuiDevRequest[] items)
    {
        var native = new NativeDevRequest[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Method = memory.Text(item.Method),
                Status = item.Status,
                Type = memory.Text(item.Type),
                Size = item.Size,
                Start = item.Start,
                Queued = item.Queued,
                Dns = item.Dns,
                Connect = item.Connect,
                Tls = item.Tls,
                Wait = item.Wait,
                Download = item.Download,
            };
        }
        return memory.Copy<NativeDevRequest>(native);
    }

    private static unsafe NativeDevTrack* PackTrack(ChartMemory memory, CuiDevTrack[] items)
    {
        var native = new NativeDevTrack[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Collapsed = item.Collapsed ? (byte)1 : (byte)0,
            };
        }
        return memory.Copy<NativeDevTrack>(native);
    }

    private static unsafe NativeDevSpan* PackSpan(ChartMemory memory, CuiDevSpan[] items)
    {
        var native = new NativeDevSpan[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Start = item.Start,
                End = item.End,
                Track = item.Track,
                Depth = item.Depth,
                ColorKey = item.ColorKey,
            };
        }
        return memory.Copy<NativeDevSpan>(native);
    }

    private static unsafe NativeDevMarker* PackMarker(ChartMemory memory, CuiDevMarker[] items)
    {
        var native = new NativeDevMarker[items.Length];
        for (int i = 0; i < items.Length; i++)
        {
            var item = items[i];
            native[i] = new()
            {
                Name = memory.Text(item.Name),
                Time = item.Time,
                Tone = item.Tone,
            };
        }
        return memory.Copy<NativeDevMarker>(native);
    }
}
