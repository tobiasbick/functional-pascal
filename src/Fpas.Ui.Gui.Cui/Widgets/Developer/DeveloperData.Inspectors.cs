using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
