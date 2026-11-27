// [Documentation](../../../../../docs/ui/gui/ai/tool-call.md)
#include "../../fpas_cui.h"

/** Draws a native call and tracks the disclosure while payload text is present. */
int fpas_cui_tool_call(cui_ctx *ui, const char *id, const char *name, int status, const char *icon,
    const char *detail, const char *input, const char *output, float seconds, int flags, int *header) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool expandable = input || output;
    const bool enabled = expandable && fpas_ai_enabled(ui, flags, key);
    cui__tc_state *state = cui_state(ui, key, sizeof *state);
    // CUI-108 (docs/future/cui-upstream-bugs.md#cui-108): a passive row has no open disclosure.
    if (!expandable) {
        state->open = false;
        state->f.t = (cui_spring){0};
        state->f.content_h = 0;
        fpas_clear_interaction(ui, key);
        if (ui->focus_next_request == key) ui->focus_next_request = 0;
    }
    const cui_tool_call_opts opts = { .status = (cui_task_status)status, .icon = icon, .detail = detail,
        .input = input, .output = output, .seconds = seconds };
    cui_push_disabled(ui, !enabled);
    cui_tool_call_ex(ui, id, name, &opts);
    cui_pop_disabled(ui);
    // CUI-064 (docs/future/cui-upstream-bugs.md#cui-064): apply the code adapter's indentation rule.
    if (expandable) {
        for (int i = root; i < ui->node_count; ++i)
            if (ui->nodes[i].paint == cui__paint_code) fpas_prepare_code_paint(ui, &ui->nodes[i]);
    }
    *header = fpas_find(ui, key);
    fpas_creative_node(ui, root, key);
    return state->open;
}
