# HTTP response status

Use a response status to summarize a received result. The gallery shows
a sample 201 response together with its time and size.
Available in **GUI**.

![HTTP status, elapsed time and size in the FPAS gallery](images/http-response-status.png)

## Example

```pascal
unit Docs.Ui.MoreDeveloperTools.HttpResponseStatus;

uses Fpas.Ui.Elements as E, Fpas.Ui.Widgets.Developer.ResponseStatuses as W;

// Displays the recorded response metrics.
// ## Parameters
// - `Status`: HTTP status code.
// - `Seconds`: Elapsed time in seconds.
// - `Bytes`: Response size in bytes.
// ## Returns
// The response status for the application view.
public function View(Status: int; Seconds: double; Bytes: double): E.Element<string>;
begin
  return W.ResponseStatus<string>('result', Status, Seconds, Bytes);
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Status / Seconds / Bytes` | Pass the code and measured values. CUI formats the status, time and size. |
| `Width / Enabled` | Content / true. |

## Interaction

Update the supplied values after receiving a response. The status line is
passive.

## See also

- [Widget API and types](../../../src/Fpas.Ui/Widgets/Developer/ResponseStatuses.fpas)
- [Gallery example](../../../examples/ui/gui/gallery/Pages/MoreDeveloper/View.fpas)
- [HTTP request bar](http-request-bar.md)
- [More developer tools](README.md)
