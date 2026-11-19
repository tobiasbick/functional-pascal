# Pipeline

Use a pipeline to show the progress of build, test or release jobs.
In the gallery, stage headers summarize the supplied job statuses.
Available in **GUI**.

![CI job stages in the FPAS gallery](images/pipeline.png)

## Example

```pascal
unit Docs.Ui.DeveloperTools.Pipeline;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.Pipelines as W;

// Displays the current CI run.
// ## Parameters
// - `Stages`: Stage names in display order.
// - `Jobs`: Jobs with stage indices and current statuses.
// - `Selected`: Selected job index, or minus one.
// ## Returns
// The pipeline with selected and clicked jobs as proposals.
public function View(Stages: array of string; Jobs: array of W.Job; Selected: int): E.Element<W.Event>;
begin
  return W.Pipeline<W.Event>('ci', Stages, Jobs, Selected,
    function(Change: W.Event): W.Event => Change);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Job.Stage` | Zero-based stage index. |
| `Job.Status` | 0 pending, 1 running, 2 passed, 3 failed, 4 skipped, 5 canceled. |
| `Job.Duration / AllowFailure` | Pass seconds, or a negative duration if unavailable. AllowFailure keeps a failed job from failing its stage. |
| `Width / Enabled` | Fill / true. Stages stack vertically on narrow panels. |

## Interaction

Store the selected job and use `Clicked` to open its output. The
application supplies updated statuses and durations as jobs progress.

## See also

- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Developer/Pipelines.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Developer/View.fpas)
- [Test explorer](../more-developer-tools/test-explorer.md)
- [Developer tools](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Developer/`
  - [Pipeline.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Developer/Pipeline.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Developer/`
  - [NativeMethods.Pipeline.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Developer/NativeMethods.Pipeline.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Developer/`
  - [CuiWindow.Pipeline.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Developer/CuiWindow.Pipeline.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/developer/`
  - [pipeline.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/pipeline.c)
  - [pipeline.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/developer/pipeline.h)
