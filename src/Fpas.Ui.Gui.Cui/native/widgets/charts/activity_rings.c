// [Documentation](../../../../../docs/ui/gui/more-charts/activity-rings.md)
#include "../../fpas_cui.h"

int fpas_cui_activity_rings(cui_ctx *ui, const char *id, const fpas_viz_item *items, int count,
    float size, float thickness, const char *center, const char *caption, int legend, int enabled, int *node) {
    const cui_rings_opts o = { .size = size, .thickness = thickness, .center = cui__frame_strdup(ui, center, -1),
        .caption = cui__frame_strdup(ui, caption, -1), .legend = legend != 0 };
    *node = ui->node_count;
    cui_push_disabled(ui, !enabled);
    int clicked = cui_activity_rings_ex(ui, id, fpas_viz_items(ui, items, count), count, &o);
    cui_pop_disabled(ui);
    ui->last_node = *node;

    /* CUI-094 (docs/future/cui-upstream-bugs.md#cui-094): an unpainted arc has no hit area. */
    cui_node *control = &ui->nodes[*node];
    if (!(control->flags & CUI_NF_ON_TOP)) return clicked;
    cui__rings_paint *paint = control->paint_data;
    cui__viz_state *state = cui_state(ui, control->id, sizeof *state + sizeof(cui_spring) * 8);
    const float radius = paint->size * 0.5f - paint->th * 0.5f - state->hover * (paint->th + 3);
    if (state->hover >= 0 && radius <= paint->th * 0.5f &&
        (!o.legend || ui->mx < cui__prev_rect(ui, control->id).x + paint->size + CUI__RING_LEGEND_GAP)) {
        state->hover = -1;
        state->hover_t = 0;
        paint->hover = -1;
        paint->tip.alpha = 0;
        control->flags &= ~(uint32_t)CUI_NF_ON_TOP;
        clicked = -1;
    }
    return clicked;
}
