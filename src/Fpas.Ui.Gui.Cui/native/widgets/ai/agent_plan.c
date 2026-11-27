// [Documentation](../../../../../docs/ui/gui/ai/agent-plan.md)
#include "../../fpas_cui.h"

typedef struct {
    cui__plan_rail *rail;
    int first_icon, last_icon, split_icon;
} fpas_plan_rail;

/** Reuses CUI's rail painter with the current icon positions. */
static void fpas_paint_plan_rail(cui_ctx *ui, cui_node *node, void *data) {
    const fpas_plan_rail *paint = data;
    cui__plan_rail *rail = paint->rail;
    const cui_rect first = ui->nodes[paint->first_icon].rect;
    const cui_rect last = ui->nodes[paint->last_icon].rect;
    rail->x0 = rail->x1 = last.x + last.w * .5f;
    rail->y0 = first.y + first.h * .5f;
    rail->y1 = last.y + last.h * .5f;
    rail->split = rail->y0;
    if (paint->split_icon >= 0) {
        const cui_rect split = ui->nodes[paint->split_icon].rect;
        rail->split = split.y + split.h * .5f;
    }
    cui__paint_plan_rail(ui, node, rail);
}

/** Draws the native plan and returns its optional pointer activation. */
int fpas_cui_agent_plan(cui_ctx *ui, const char *id, const cui_plan_step *steps, int count, const char *title, int flags, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_agent_plan_opts opts = { .title = title };
    cui_push_disabled(ui, !enabled);
    const int picked = cui_agent_plan_ex(ui, id, steps, count, &opts);
    cui_pop_disabled(ui);
    int split = -1;
    for (int i = 0; i < count; ++i) {
        parts[i] = fpas_find(ui, fpas_well_part(key ^ 0x57E9u, i));
        if (steps[i].status == CUI_TASK_DONE || steps[i].status == CUI_TASK_SKIPPED || steps[i].status == CUI_TASK_RUNNING)
            split = ui->nodes[parts[i]].first;
    }
    if (count > 1) {
        /* CUI-111 (docs/future/cui-upstream-bugs.md#cui-111): native paint data uses the preceding layout. */
        const int heading = ui->nodes[root].first;
        cui_node *list = &ui->nodes[ui->nodes[heading].next];
        fpas_plan_rail *paint = cui__frame_alloc(ui, sizeof *paint);
        *paint = (fpas_plan_rail){ list->paint_data, ui->nodes[parts[0]].first,
            ui->nodes[parts[count - 1]].first, split };
        list->paint = fpas_paint_plan_rail;
        list->paint_data = paint;
    }
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
