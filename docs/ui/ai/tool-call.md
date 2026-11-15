# Tool call

The gallery shows search and file-read calls. Expanding the search call
reveals its input and output JSON.
Available in **GUI**.

![Expanded search tool call with input and output in the FPAS gallery](images/tool-call.png)

## Example

```pascal
unit Docs.Ui.Ai.ToolCall;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Ai.ToolCalls as W, Fpas.Ui.Widgets.Feedback.TaskStates as T;

// Shows the result of a completed search.
// ## Parameters
// - `Input`: Recorded input display text.
// - `Output`: Recorded output display text.
// - `Seconds`: Elapsed seconds.
// ## Returns
// The disclosure for a recorded tool call.
public function View(Input: string; Output: string; Seconds: float): E.Element<string>;
begin
  return W.ToolCall<string>('search', 'web_search',
    new W.Options(Status := T.Status.Done, Input := Input, Output := Output, Seconds := Seconds));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Status / Seconds` | Pending / 0. Pass the current task state and timing. |
| `Icon / Detail / Input / Output` | All empty. Detail summarizes the call. Input and Output appear when expanded. |
| `Enabled` | true. Allows opening payload details. |

## Interaction

Open a call with payload text to inspect its details. Update status and
payloads as your application receives them. The widget renders the tool
call that the application supplies.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Ai/ToolCalls.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [Agent status](agent-status.md)
- [AI](README.md)
