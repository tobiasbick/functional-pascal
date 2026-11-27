// [Documentation](../../../../../docs/ui/gui/developer-tools/dependency-graph.md)
#include "../../fpas_cui.h"

/** Releases CUI's wheel transform along with the graph. */
static void fpas_dependency_graph_forget(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, (key ^ 0x6A90u) ^ CUI__WHEEL_XFORM);
}

/** Builds CUI's dependency graph and exposes its node observations. */
void fpas_cui_dependency_graph(cui_ctx *ui, const char *id, const cui_graph_node *vertices, int vertices_count,
    const cui_graph_edge *edges, int edges_count, int selection, float height, int top_down,
    int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0x6A9Fu, wheel = key ^ 0x6A90u;
    const bool allowed = (flags & 1) && !cui_is_disabled(ui);
    const bool retained = selection == -2;
    cui__gv_state *state = cui__state_lazy(ui, slot, sizeof *state, false);
    const int previous = retained ? (state ? state->own_sel : -1) : selection;
    fpas_own(ui, key, slot, wheel, 0);
    fpas_on_forget(ui, key, fpas_dependency_graph_forget);
    if (allowed && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = key;
    if (!allowed || !ui->window_focused) {
        /* CUI-130 (docs/future/cui-upstream-bugs.md#cui-130): cancel a pending native click or pan before disable. */
        if (state) state->mode = 0;
        if (ui->scroll_target == wheel) ui->scroll_target = 0;
    }
    *out = fpas_dev_empty();
    const cui_graph_opts options = { .width = fpas_extent(width_kind, width), .height = height,
        .top_down = top_down != 0, .selected = retained ? NULL : &selection };
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = allowed && fpas_dev_save_keys(ui, key, keys);
    cui_push_disabled(ui, !allowed);
    const cui_graph_result result = cui_graph_view_ex(ui, id, vertices, vertices_count, edges, edges_count, &options);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof ui->kpressed);
    if (!allowed) ui->nodes[ui->nodes[root].first].paint = NULL;
    fpas_creative_node(ui, root, key);
    state = cui__state_lazy(ui, slot, sizeof *state, false);
    out->index = retained ? state->own_sel : selection;
    out->flags = out->index != previous;
    out->action = result.activated;
    out->hover = result.hovered;
    out->count = result.clicked;
}
