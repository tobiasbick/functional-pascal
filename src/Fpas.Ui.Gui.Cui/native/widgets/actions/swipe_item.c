// [Documentation](../../../../../docs/ui/gui/actions/swipe-actions.md)
#include "../../fpas_cui.h"

/** Opens the native swipe row and reports revealed actions for inspection. */
int fpas_cui_swipe_begin(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, const int *tones, int left_count, int right_count, int enabled,
    const char *icon, int *nodes) {
    /* CUI-032: an unmatched pop after a failed push removes an enclosing identity.
       See docs/future/cui-upstream-bugs.md#cui-032. */
    if (ui->id_depth >= CUI_ID_STACK || ui->parent_depth > CUI_PARENT_STACK - 4 || ui->disabled_n >= 32)
        return -1;
    const int count = left_count + right_count;
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_swipe_action *actions = count ? cui__frame_alloc(ui, (size_t)count * sizeof *actions) : NULL;
    for (int i = 0; i < count; ++i)
        actions[i] = (cui_swipe_action){ icons[i], labels[i], (cui_tone)tones[i] };
    if (!allowed) fpas_forget_state(ui, key);
    cui_push_disabled(ui, !allowed);
    cui_swipe_item_begin(ui, id, actions, allowed ? left_count : 0,
        right_count ? actions + left_count : NULL, allowed ? right_count : 0);
    cui_pop_disabled(ui);
    const cui__swipe_state *state = cui_state(ui, key, sizeof *state);
    for (int i = 0; i < count; ++i) {
        const bool left = i < left_count;
        const int index = left ? i : 100 + i - left_count;
        const bool revealed = left ? state->x.x > 1 : state->x.x < -1;
        nodes[i + 1] = allowed && revealed ? fpas_find(ui, cui__hash(&index, sizeof index, key)) : -1;
    }
    nodes[0] = fpas_find(ui, key);
    fpas_own(ui, key, key ^ 0x5F1Fu, 0, 0);
    if (icon) cui_icon(ui, icon, 16, CUI_COLOR_TEXT_MUTED);
    return nodes[0];
}

int fpas_cui_swipe_end(cui_ctx *ui, int node, const char *detail) {
    if (detail) cui_text(ui, detail, .size = 12, .color = CUI_COLOR_TEXT_MUTED);
    const int result = cui_swipe_item_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
    return result;
}
