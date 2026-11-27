// [Documentation](../../../../../docs/ui/gui/more-developer-tools/api-explorer.md)
#include "../../fpas_cui.h"

/** Checks the tag fold and the current path and summary filter. */
static bool fpas_api_visible(const cui__ax_state *state, const cui_api_endpoint *endpoint) {
    const uint32_t tag = cui__hash_str(cui__ax_tag(endpoint), 0xA9u);
    for (int i = 0; i < state->nfolded; ++i)
        if (state->folded[i] == tag) return false;
    return !state->query[0]
        || cui__contains_ci(endpoint->path ? endpoint->path : "", state->query)
        || cui__contains_ci(endpoint->summary ? endpoint->summary : "", state->query);
}

/** Finds a visible neighbor in the native first-seen tag order. */
static int fpas_api_move(cui_ctx *ui, const cui__ax_state *state, const cui_api_endpoint *endpoints,
    int count, int selection, bool up) {
    bool *seen = cui__frame_alloc(ui, (size_t)count * sizeof *seen);
    memset(seen, 0, (size_t)count * sizeof *seen);
    int first = -1, previous = -1;
    bool passed_selection = false;
    for (int i = 0; i < count; ++i) {
        if (seen[i]) continue;
        const char *tag = cui__ax_tag(&endpoints[i]);
        for (int j = i; j < count; ++j) {
            if (seen[j] || strcmp(tag, cui__ax_tag(&endpoints[j]))) continue;
            seen[j] = true;
            if (!fpas_api_visible(state, &endpoints[j])) continue;
            if (first < 0) first = j;
            if (passed_selection) return j;
            if (selection == j) {
                if (up) return previous >= 0 ? previous : j;
                passed_selection = true;
            }
            previous = j;
        }
    }
    return passed_selection ? selection : first;
}

/** CUI-139 (docs/future/cui-upstream-bugs.md#cui-139): leave keys with visible endpoints. */
static void fpas_api_keys(cui_ctx *ui, cui__ax_state *state, const cui_api_endpoint *endpoints,
    int count, int *selection) {
    if (!state || !endpoints || count <= 0) return;
    const bool down = cui_key_pressed(ui, CUI_KEY_DOWN), up = cui_key_pressed(ui, CUI_KEY_UP);
    if (state->nfolded && (up || down)) {
        if (down) *selection = fpas_api_move(ui, state, endpoints, count, *selection, false);
        if (up) *selection = fpas_api_move(ui, state, endpoints, count, *selection, true);
        ui->kpressed[CUI_KEY_DOWN] = ui->kpressed[CUI_KEY_UP] = 0;
        ui->focus_visible = true;
        state->scroll_to_sel = *selection >= 0;
    }
    const bool selected_visible = *selection >= 0 && *selection < count
        && fpas_api_visible(state, &endpoints[*selection]);
    if (!selected_visible && (state->nfolded || state->query[0])) {
        ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    }
}

/** Retires the filter field along with the API explorer's list and query. */
static void fpas_api_explorer_retire(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, cui__hash_str("filter", key));
}

/** Draws the explorer with model-owned or native selection. */
void fpas_cui_api_explorer(cui_ctx *ui, const char *id, const cui_api_endpoint *endpoints, int endpoints_count, int selection, float height, int width_kind, float width, int flags, fpas_dev_result *out) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id slot = key ^ 0xA91Eu, list = key ^ 0xA91Fu;
    const cui_id scroll = cui__hash_str("list", key);
    const bool enabled = fpas_ai_enabled(ui, flags, list);
    const bool retained = selection == -2;
    cui__ax_state *state = cui__state_lazy(ui, slot, sizeof *state, false);
    const int previous = retained ? (state ? state->own_sel : -1) : selection;
    fpas_document_scroll_input(ui, scroll, enabled);
    fpas_own(ui, key, slot, scroll, 0);
    fpas_on_forget(ui, key, fpas_api_explorer_retire);
    fpas_owned_state *owner = cui_state(ui, fpas_owner_key(key), sizeof *owner);
    owner->focus = list;
    uint8_t keys[CUI_KEY_COUNT];
    bool muted = enabled && fpas_dev_save_keys(ui, list, keys);
    if (enabled && !muted && !ui->edit_prev && (ui->focus == list || ui->focus_next_request == list)) {
        memcpy(keys, ui->kpressed, sizeof keys);
        int *selected = retained ? (state ? &state->own_sel : NULL) : &selection;
        if (selected) fpas_api_keys(ui, state, endpoints, endpoints_count, selected);
        muted = true;
    }
    *out = fpas_dev_empty();
    const cui_api_explorer_opts o = {
        .width = fpas_extent(width_kind, width), .height = height, .selected = retained ? NULL : &selection
    };
    cui_push_disabled(ui, !enabled);
    const cui_api_explorer_result r = cui_api_explorer_ex(ui, id, endpoints, endpoints_count, &o);
    cui_pop_disabled(ui);
    if (muted) memcpy(ui->kpressed, keys, sizeof keys);
    const int viewport = fpas_find(ui, scroll);
    if (viewport >= 0) ui->nodes[viewport].paint = enabled ? fpas_document_scroll_layout : fpas_disabled_document_scroll_layout;
    if (ui->node_count == root) cui__leaf(ui, key, CUI_FIT, CUI_FIT);
    fpas_dev_watch_width(ui, root, key);
    fpas_creative_node(ui, root, key);
    out->index = r.selected;
    out->action = r.try_it;
    out->flags = r.selected != previous;
}
