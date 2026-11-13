#include "../fpas_cui.h"

int fpas_cui_card_stack_begin(cui_ctx *ui, const char *id, int count,
    int width_kind, float width, float height, int visible, int enabled, int focus) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), top = key ^ 0x70Bu;
    fpas_own(ui, key, top, 0, 0);
    const bool accept = enabled && count > 0 && !cui_is_disabled(ui);
    if (accept && focus && count > 0) ui->focus_next_request = top;
    if (!accept) {
        cui__stack_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) { state->dragging = false; state->flying = false; }
    }
    const cui_card_stack_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .visible = visible };
    cui_push_disabled(ui, !accept);
    cui_card_stack_begin(ui, id, count, &options);
    cui_pop_disabled(ui);
    return root;
}

void fpas_cui_card_stack_item_begin(cui_ctx *ui, int depth) { cui_card_stack_item_begin(ui, depth); }
void fpas_cui_card_stack_item_end(cui_ctx *ui) { cui_card_stack_item_end(ui); }

int fpas_cui_card_stack_end(cui_ctx *ui, int root, int enabled, int *top) {
    const cui_id key = ui->nodes[root].id;
    const int result = cui_card_stack_end(ui);
    *top = fpas_find(ui, key ^ 0x70Bu);
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
