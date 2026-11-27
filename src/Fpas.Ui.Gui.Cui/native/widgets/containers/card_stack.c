// [Documentation](../../../../../docs/ui/gui/social-and-media/card-stack.md)
#include "../../fpas_cui.h"

int fpas_cui_card_stack_begin(cui_ctx *ui, const char *id, int count,
    int width_kind, float width, float height, int visible, int enabled, int focus) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), top = key ^ 0x70Bu;
    fpas_own(ui, key, top, 0, 0);
    /* [CUI-066](../../../../../docs/future/cui-upstream-bugs.md#cui-066): an empty deck has no focus target. */
    const bool accept = enabled && count > 0 && !cui_is_disabled(ui);
    if (accept && focus) ui->focus_next_request = top;
    const uint8_t left = ui->kpressed[CUI_KEY_LEFT], right = ui->kpressed[CUI_KEY_RIGHT];
    cui__stack_state *state = (!accept || left || right)
        ? cui__state_lazy(ui, key, sizeof *state, false) : NULL;
    if (!accept && state) {
        state->dragging = false;
        state->flying = false;
    }
    /* [CUI-067](../../../../../docs/future/cui-upstream-bugs.md#cui-067): a drag owns the swipe through release. */
    const bool suppress_keys = accept && (left || right) &&
        ((state && state->dragging) || (ui->hot == top && ui->mpressed[0]));
    if (suppress_keys) ui->kpressed[CUI_KEY_LEFT] = ui->kpressed[CUI_KEY_RIGHT] = 0;
    const cui_card_stack_opts options = { .width = fpas_extent(width_kind, width),
        .height = height, .visible = visible };
    cui_push_disabled(ui, !accept);
    cui_card_stack_begin(ui, id, count, &options);
    cui_pop_disabled(ui);
    if (suppress_keys) {
        ui->kpressed[CUI_KEY_LEFT] = left;
        ui->kpressed[CUI_KEY_RIGHT] = right;
    }
    return root;
}

void fpas_cui_card_stack_item_begin(cui_ctx *ui, int depth) { cui_card_stack_item_begin(ui, depth); }
void fpas_cui_card_stack_item_end(cui_ctx *ui) { cui_card_stack_item_end(ui); }

int fpas_cui_card_stack_end(cui_ctx *ui, int root, int enabled, int *top) {
    const cui_id key = ui->nodes[root].id, top_key = key ^ 0x70Bu;
    const int result = cui_card_stack_end(ui);
    *top = fpas_find(ui, top_key);
    /* [CUI-071](../../../../../docs/future/cui-upstream-bugs.md#cui-071): show the top card's keyboard focus. */
    if (enabled && *top >= 0 && ui->focus_visible && ui->focus == top_key)
        cui__focus_ring(ui, *top, 1);
    fpas_creative_node(ui, root, key);
    return enabled ? result : 0;
}
