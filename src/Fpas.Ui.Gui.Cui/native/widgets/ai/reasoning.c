#include "../../fpas_cui.h"

int fpas_cui_reasoning_begin(cui_ctx *ui, const char *id, int thinking, float seconds, const char *title, int flags, int *nodes) {
    nodes[0] = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = fpas_ai_enabled(ui, flags, key);
    const cui_reasoning_opts opts = { .thinking = thinking != 0, .seconds = seconds, .title = title };
    cui_push_disabled(ui, !enabled);
    const bool built = cui_reasoning_begin(ui, id, &opts);
    nodes[1] = fpas_find(ui, key);
    if (!built) {
        cui_pop_disabled(ui);
        fpas_creative_node(ui, nodes[0], key);
        return 0;
    }
    const cui__rs_state *state = cui_state(ui, key, sizeof *state);
    return state->open ? 3 : 1;
}

void fpas_cui_reasoning_end(cui_ctx *ui, int root, int head) {
    cui_reasoning_end(ui);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, root, ui->nodes[head].id);
}
