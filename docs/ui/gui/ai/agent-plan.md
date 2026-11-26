# Agent plan

The gallery shows a research plan with completed, running and pending steps.
Available in **GUI**.

![Research plan with step states in the FPAS gallery](images/agent-plan.png)

## Example

```pascal
unit Docs.Ui.Ai.AgentPlan;

uses Fpas.Ui.Elements as E;
uses Fpas.Ui.Widgets.Ai.AgentPlans as W;

// Shows the current research plan.
// ## Parameters
// - `Steps`: Ordered steps with their current states.
// ## Returns
// The plan with selected step indices as messages.
public function View(Steps: array of W.Step): E.Element of int;
begin
  return W.AgentPlan of int('research-plan', Steps, function(Index: int): int => Index,
    new W.Options(Title := 'Research plan'));
end function;

end unit;
```

## Options and defaults

| Option | Default or requirement |
| --- | --- |
| `Step.Title / Detail / Status` | Supply the title. Detail defaults to empty and Status to Pending. |
| `Title / Enabled` | Empty / true. Empty Title uses the native heading. |
| `Changed` | Pass a mapper to handle step activation, or null for a passive plan. |

## Interaction

Update each step in the model as work progresses. When activation is
enabled, use the proposed zero-based index to open step details.

## See also

- [CUI widget inventory](../../../cui-widget-inventory.md#cui_agent_plan)
- [Widget API and types](../../../../src/Fpas.Ui/Widgets/Ai/AgentPlans.fpas)
- [Gallery example](../../../../examples/ui/gui/gallery/Pages/Ai/View.fpas)
- [AI](README.md)

## Source files

- `src/Fpas.Ui.Gui/Internal/Widgets/Ai/`
  - [AgentPlan.fpas](../../../../src/Fpas.Ui.Gui/Internal/Widgets/Ai/AgentPlan.fpas)
- `src/Fpas.Ui.Gui.Cui/Interop/Ai/`
  - [NativeMethods.AgentPlan.cs](../../../../src/Fpas.Ui.Gui.Cui/Interop/Ai/NativeMethods.AgentPlan.cs)
- `src/Fpas.Ui.Gui.Cui/Widgets/Ai/`
  - [CuiWindow.AgentPlan.cs](../../../../src/Fpas.Ui.Gui.Cui/Widgets/Ai/CuiWindow.AgentPlan.cs)
- `src/Fpas.Ui.Gui.Cui/native/widgets/ai/`
  - [agent_plan.c](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/agent_plan.c)
  - [agent_plan.h](../../../../src/Fpas.Ui.Gui.Cui/native/widgets/ai/agent_plan.h)
