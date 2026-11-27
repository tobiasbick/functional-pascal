// [Documentation](../../../../../docs/ui/gui/more-developer-tools/feature-flags.md)
#include "../../fpas_cui.h"

/** Identifies the cell whose rollout popover was opened. */
static cui_id fpas_feature_flag_cell(cui_id key, const cui__ff_state *state) {
    if (!state || state->pop_flag < 0) return 0;
    const cui_id row = cui__hash(&state->pop_flag, sizeof state->pop_flag, key);
    return cui__hash(&state->pop_env, sizeof state->pop_env, row);
}

/** CUI-141 (docs/future/cui-upstream-bugs.md#cui-141): follow the changed share's button identity. */
static void fpas_feature_flag_anchor(cui_ctx *ui, cui_id cell, cui__ff_state *state,
    const cui_feature_flag *flags, int count, int environment_count, bool enabled) {
    if (!cell) return;
    cui_popup *popup = cui__popup_get(ui, cui__hash_str("share", cell), false);
    if (!popup || !popup->open) return;
    const int flag = state->pop_flag, env = state->pop_env;
    if (!enabled || flag >= count || env >= environment_count || !flags[flag].on[env]) {
        popup->open = false;
        return;
    }
    const int share = cui__ff_share(&flags[flag], env);
    char label[6];
    snprintf(label, sizeof label, "%d%%", share);
    const cui_id anchor = cui__hash_str(label, cell);
    if (popup->anchor && ui->focus == popup->anchor) ui->focus = anchor;
    popup->anchor = anchor;
    state->rollout = (float)share;
}

/** Retires the editor for the filter and navigation state for the last popover. */
static void fpas_feature_flags_retire(cui_ctx *ui, cui_id key) {
    const cui__ff_state *state = cui__state_lazy(ui, key ^ 0xF1A6u, sizeof *state, false);
    const cui_id cell = fpas_feature_flag_cell(key, state);
    if (cell) fpas_forget_state(ui, cui__hash_str("share", cell) ^ 0x4E4156u);
    fpas_forget_state(ui, cui__hash_str("filter", key));
}

/** Draws flags and translates the changed environment cell. */
void fpas_cui_feature_flags(cui_ctx *ui, const char *id, cui_feature_flag *flags, int count, const char *const *environments, int environment_count, float height, int width_kind, float width, int enabled, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id slot = key ^ 0xF1A6u, rows = cui__hash_str("rows", key);
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui__ff_state *state = cui__state_lazy(ui, slot, sizeof *state, false);
    const cui_id previous_cell = fpas_feature_flag_cell(key, state);
    fpas_feature_flag_anchor(ui, previous_cell, state, flags, flags ? count : 0,
        environments ? CUI_MIN(environment_count, CUI_FLAG_ENVS) : 0, allowed);
    fpas_document_scroll_input(ui, rows, allowed);
    *out = fpas_dev_empty();
    cui_push_disabled(ui, !allowed);
    const cui_feature_flags_opts o = { .width = fpas_extent(width_kind, width), .height = height };
    const cui_feature_flags_result r = cui_feature_flags_ex(ui, id, flags, count, environments, environment_count, &o);
    cui_pop_disabled(ui);
    const int viewport = fpas_find(ui, rows);
    if (viewport >= 0) ui->nodes[viewport].paint = allowed ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    state = cui__state_lazy(ui, slot, sizeof *state, false);
    const cui_id cell = fpas_feature_flag_cell(key, state);
    const cui_id popup = cell ? cui__hash_str("share", cell) : 0;
    cui_popup *layer = popup ? cui__popup_get(ui, popup, false) : NULL;
    if (layer && layer->open && fpas_find(ui, layer->anchor) < 0) {
        layer->open = false;
        cui_request_redraw(ui);
    }
    fpas_own(ui, key, slot, rows, popup);
    fpas_on_forget(ui, key, fpas_feature_flags_retire);
    if (previous_cell && previous_cell != cell)
        fpas_forget_state(ui, cui__hash_str("share", previous_cell) ^ 0x4E4156u);
    out->index = r.flag;
    out->action = r.env;
}
