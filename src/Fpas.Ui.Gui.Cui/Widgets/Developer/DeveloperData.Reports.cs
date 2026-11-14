using Fpas.Ui.Gui.Cui.Interop.Developer;
using Fpas.Ui.Gui.Cui.Widgets.Charts;
using Fpas.Ui.Gui.Cui.Widgets.Developer;
using Fpas.Ui.Gui.Cui.Interop;

namespace Fpas.Ui.Gui.Cui;

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
