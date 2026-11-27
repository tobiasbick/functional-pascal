// [Documentation](../../../../../docs/ui/gui/ai/prompt-suggestions.md)
#include "../../fpas_cui.h"

/** Maps an item position to the identity used by native suggestions. */
static cui_id fpas_suggestion_part(cui_id key, int index) {
    return fpas_well_part(key ^ 0x9A0u, index);
}

/** Releases suggestion animation state and input ownership when the grid is removed. */
static void fpas_suggestions_forget(cui_ctx *ui, cui_id key) {
    const int *count = cui__state_lazy(ui, key ^ 0x9A05u, sizeof *count, false);
    if (count) for (int i = 0; i < *count; ++i) fpas_forget_state(ui, fpas_suggestion_part(key, i));
}

/** Draws the native suggestions and returns an enabled card activation. */
int fpas_cui_prompt_suggestions(cui_ctx *ui, const char *id, const cui_prompt_suggestion *items,
    int count, int columns, int flags, int focus, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const cui_id count_key = key ^ 0x9A05u;
    int *previous = cui_state(ui, count_key, sizeof *previous);
    for (int i = count; i < *previous; ++i) fpas_forget_state(ui, fpas_suggestion_part(key, i));
    *previous = count;
    fpas_own(ui, key, count_key, 0, 0);
    fpas_on_forget(ui, key, fpas_suggestions_forget);

    if (enabled && focus >= 0 && focus < count) ui->focus_next_request = fpas_suggestion_part(key, focus);
    const cui_prompt_suggestions_opts opts = { .columns = columns };
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): covered cards still report keyboard activation. */
    const bool covered = (enter || space) && cui__keys_covered(ui);
    cui_push_disabled(ui, !enabled);
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    const int picked = cui_prompt_suggestions_ex(ui, id, items, count, &opts);
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }
    cui_pop_disabled(ui);
    for (int i = 0; i < count; ++i) parts[i] = fpas_find(ui, fpas_suggestion_part(key, i));
    fpas_creative_node(ui, root, key);
    return enabled ? picked : -1;
}
