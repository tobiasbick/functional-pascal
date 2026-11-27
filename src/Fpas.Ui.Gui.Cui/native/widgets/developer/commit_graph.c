// [Documentation](../../../../../docs/ui/gui/developer-tools/commit-graph.md)
#include "../../fpas_cui.h"

typedef struct {
    void (*native)(cui_ctx *, cui_node *, void *);
    void *data;
    bool narrow;
} fpas_commit_paint;

/** Requests a new column layout when the final width crosses the native breakpoint. */
static void fpas_paint_commit_graph(cui_ctx *ui, cui_node *node, void *data) {
    const fpas_commit_paint *paint = data;
    /* CUI-132 (docs/future/cui-upstream-bugs.md#cui-132): CUI uses the preceding width. */
    if ((node->rect.w > 0 && node->rect.w < CUI__CG_NARROW) != paint->narrow) cui_request_redraw(ui);
    if (paint->native) paint->native(ui, node, paint->data);
}

/** Builds CUI's history and exposes its row observations. */
void fpas_cui_commit_graph(cui_ctx *ui, const char *id, const cui_commit *commits, int commits_count,
    int selection, float height, int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), slot = key ^ 0xC0117u, list = key ^ 0xC01Du;
    cui_push_id(ui, id);
    const cui_id rows = cui_make_id(ui, "rows");
    cui_pop_id(ui);
    const bool allowed = (flags & 1) && !cui_is_disabled(ui);
    const bool retained = selection == -2;
    const cui__cg_state *state = cui__state_lazy(ui, slot, sizeof *state, false);
    const int previous = retained ? (state ? state->own_sel : -1) : selection;
    const float previous_width = cui__prev_rect(ui, key).w;
    fpas_own(ui, key, slot, rows, 0);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = list;
    if (allowed && (flags & 2) && !cui__keys_covered(ui)) ui->focus_next_request = list;
    if (!allowed && ui->scroll_target == rows) ui->scroll_target = 0;

    *out = fpas_dev_empty();
    const cui_commit_graph_opts options = { .width = fpas_extent(width_kind, width), .height = height,
        .selected = retained ? NULL : &selection };
    uint8_t keys[CUI_KEY_COUNT];
    const bool muted = allowed && fpas_dev_save_keys(ui, list, keys);
    cui_push_disabled(ui, !allowed);
    const cui_commit_graph_result result = cui_commit_graph_ex(ui, id, commits, commits_count, &options);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof ui->kpressed);
    cui_node *panel = &ui->nodes[root];
    fpas_commit_paint *paint = cui__frame_alloc(ui, sizeof *paint);
    *paint = (fpas_commit_paint){ panel->paint, panel->paint_data, previous_width > 0 && previous_width < CUI__CG_NARROW };
    panel->paint = fpas_paint_commit_graph;
    panel->paint_data = paint;
    const int viewport = fpas_find(ui, rows);
    if (!allowed && viewport >= 0) ui->nodes[viewport].flags &= ~(CUI_NF_SCROLL_X | CUI_NF_SCROLL_Y);
    fpas_creative_node(ui, root, key);
    state = cui__state_lazy(ui, slot, sizeof *state, false);
    out->index = retained ? state->own_sel : selection;
    out->flags = out->index != previous;
    out->action = result.activated;
    out->hover = result.hovered;
    out->count = result.clicked;
}
