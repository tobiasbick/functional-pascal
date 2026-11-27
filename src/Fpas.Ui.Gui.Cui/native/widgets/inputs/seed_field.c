// [Documentation](../../../../../docs/ui/gui/creative-controls/seed-field.md)
#include "../../fpas_cui.h"

/** Draws a controlled seed field and writes roll or lock proposals. */
int fpas_cui_seed_field(cui_ctx *ui, const char *id,
    const char *label, uint32_t *seed, int *locked, const char *display, const char *random, const char *fixed, int flags, int focus, int *parts) {
    const int node = ui->node_count;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const char *literal = label;
    label = fpas_well_label(ui, label, id);
    const cui_id key = cui_make_id(ui, label);
    const cui_id dice = key ^ 0xD1CEu, lock = key ^ CUI__WELL_TOOL_SALT;
    /* CUI-159 (docs/future/cui-upstream-bugs.md#cui-159): stop an existing roll when motion is reduced. */
    if (ui->theme.motion <= 0) {
        cui__seed_state *state = cui__state_lazy(ui, key, sizeof *state, false);
        if (state) state->rolling = false;
    }
    bool current = locked && *locked != 0;
    /* CUI-049 (docs/future/cui-upstream-bugs.md#cui-049): release a hidden lock. */
    if (!locked) fpas_clear_interaction(ui, lock);
    if (enabled && focus == 0 && !current) ui->focus_next_request = dice;
    if (enabled && focus == 1 && locked) ui->focus_next_request = lock;
    const cui_seed_field_opts options = { .value = display, .random_text = random, .fixed_text = fixed,
        .plain = (flags & 8) != 0 };
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): keep activation keys available to the popup. */
    const bool covered = enabled && cui__keys_covered(ui);
    const fpas_well_keys keys = fpas_well_pressed(ui);
    cui_push_disabled(ui, !enabled);
    if (covered) fpas_well_set_keys(ui, (fpas_well_keys){0});
    const int result = cui_seed_field_ex(ui, label, seed, locked ? &current : NULL, &options);
    if (covered) fpas_well_set_keys(ui, keys);
    cui_pop_disabled(ui);
    if (locked) *locked = current;
    parts[0] = fpas_find(ui, dice);
    parts[1] = locked ? fpas_find(ui, lock) : -1;
    fpas_well_literal(ui, node, literal);
    fpas_creative_node(ui, node, key);
    return enabled ? result : 0;
}
