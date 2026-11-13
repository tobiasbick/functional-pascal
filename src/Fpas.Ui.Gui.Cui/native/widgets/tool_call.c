#include "../fpas_cui.h"

int fpas_cui_tool_call(cui_ctx *ui, const char *id, const char *name, int status, const char *icon,
    const char *detail, const char *input, const char *output, float seconds, int flags, int *header) {
    const int root = ui->node_count;
    const bool animating = ui->animating;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    const cui_tool_call_opts opts = { .status = (cui_task_status)status, .icon = icon, .detail = detail,
        .input = input, .output = output, .seconds = seconds };
    cui_push_disabled(ui, !enabled);
    cui_tool_call_ex(ui, id, name, &opts);
    fpas_ai_freeze_status(ui, root, animating);
    cui_pop_disabled(ui);
    *header = fpas_find(ui, key);
    fpas_creative_node(ui, root, key);
    const cui__tc_state *state = cui_state(ui, key, sizeof *state);
    return state->open;
}
