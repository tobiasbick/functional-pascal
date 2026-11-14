#include "../../fpas_cui.h"

int fpas_cui_agent_plan(cui_ctx *ui, const char *id, const cui_plan_step *steps, int count, const char *title, int flags, int *parts) {
    const int root = ui->node_count;
    const bool animating = ui->animating;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_agent_plan_opts opts = { .title = title };
    cui_push_disabled(ui, !enabled);
    const int picked = cui_agent_plan_ex(ui, id, steps, count, &opts);
    fpas_ai_freeze_status(ui, root, animating);
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x57E9u, i));
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
