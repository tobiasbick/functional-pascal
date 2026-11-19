// [Documentation](../../../../../docs/ui/gui/disclosure/link.md)
#include "../../fpas_cui.h"

static void fpas_paint_link(cui_ctx *ui, cui_node *node, void *data) {
    cui__paint_underline(ui, node, data);
    const cui__btn_state *state = cui__state_lazy(ui, node->id, sizeof *state, false);
    if (state) cui__draw_focus_ring(ui, node->rect, node->radius, state->focus);
}

int fpas_cui_link(cui_ctx *ui, const char *id, const char *label, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id) ^ 0x11AC0009u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    const bool clicked = cui_link(ui, id);
    cui_pop_disabled(ui);

    *node = ui->last_node;
    cui_node *link = &ui->nodes[*node];
    fpas_text(ui, *node, label);
    if (!allowed) link->opacity = CUI__DISABLED;
    /* Draw the focus ring alongside the link's hover underline. */
    if (allowed) link->paint = fpas_paint_link;
    return allowed && clicked;
}
