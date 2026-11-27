// [Documentation](../../../../../docs/ui/gui/files/file-browser.md)
#include "../../fpas_cui.h"

typedef struct { int revision, maximum; bool hidden, custom; } fpas_file_listing_options;
typedef struct { cui_file_list_fn list; void *user; int maximum; } fpas_file_listing_source;
static const cui_key fpas_file_navigation_keys[] = { CUI_KEY_F5, CUI_KEY_LEFT, CUI_KEY_RIGHT, CUI_KEY_ESCAPE };

/** Masks native navigation shortcuts while the browser or picker is disabled. */
static unsigned fpas_file_mask_keys(cui_ctx *ui, bool enabled) {
    if (enabled) return 0;
    // CUI-127 (docs/future/cui-upstream-bugs.md#cui-127): F5 and history are global to the browser.
    unsigned pressed = 0;
    for (size_t i = 0; i < CUI_COUNTOF(fpas_file_navigation_keys); ++i) {
        if (ui->kpressed[fpas_file_navigation_keys[i]]) pressed |= 1u << i;
        ui->kpressed[fpas_file_navigation_keys[i]] = 0;
    }
    return pressed;
}

/** Restores shortcuts for the controls that can accept them. */
static void fpas_file_restore_keys(cui_ctx *ui, unsigned pressed) {
    if (!pressed) return;
    for (size_t i = 0; i < CUI_COUNTOF(fpas_file_navigation_keys); ++i)
        if (pressed & (1u << i)) ui->kpressed[fpas_file_navigation_keys[i]] = 1;
}

/** Applies the current limit even when CUI retains a larger listing buffer. */
static int fpas_file_list_limited(void *user, const char *path, cui_file_entry *entries, int capacity,
    char *names, size_t name_capacity, bool hidden) {
    const fpas_file_listing_source *source = user;
    return source->list(source->user, path, entries, CUI_MIN(capacity, source->maximum), names, name_capacity, hidden);
}

/** Clears a selection if its row no longer passes the current filters. */
static bool fpas_file_clear_hidden_selection(cui__fb_state *state, const char *filter, bool folders_only) {
    // CUI-124 (docs/future/cui-upstream-bugs.md#cui-124): only visible entries are selectable.
    if (!state || state->sel < 0 || state->sel >= state->n) return false;
    const cui_file_entry *entry = &state->ent[state->sel];
    if ((entry->dir || (!folders_only && cui__glob(filter, entry->name))) && cui__contains_ci(entry->name, state->query))
        return false;
    state->sel = -1;
    state->scroll_to_sel = false;
    return true;
}

/** Releases browser state owned by the native list and its toolbar. */
static void fpas_file_browser_forget(cui_ctx *ui, cui_id key) {
    const cui_id rows = cui__hash_str("rows", key);
    const cui_id path = cui__hash_str("path", key);
    fpas_forget_state(ui, rows);
    fpas_forget_state(ui, path ^ 0x9A7B0u);
    fpas_forget_state(ui, path ^ 0xED17u);
    fpas_forget_state(ui, cui__hash_str("filter", key));
}

/** Draws the native browser with the current listing options and selection. */
int fpas_cui_file_browser(cui_ctx *ui, const char *id, char *folder, int capacity,
    int width_kind, float width, int height_kind, float height, const char *filter,
    const char *const *places, int place_count, cui_file_list_fn source, void *user, int maximum,
    int revision, int flags, int focus, const char **selected, const char **picked, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), list = key ^ 0x115Du;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus) ui->focus_next_request = list;
    const cui_id listing_key = key ^ 0xF11Eu;
    fpas_file_listing_options *seen = cui_state(ui, listing_key, sizeof *seen);
    cui__fb_state *previous = cui__state_lazy(ui, key, sizeof *previous, false);
    const fpas_file_listing_options current = { revision, maximum > 0 ? maximum : 20000, (flags & 4) != 0, source != NULL };
    // CUI-121 (docs/future/cui-upstream-bugs.md#cui-121): the native cache keys only on the path.
    if (previous && (seen->revision != current.revision || seen->maximum != current.maximum ||
        seen->hidden != current.hidden || seen->custom != current.custom)) previous->reload = true;
    *seen = current;
    fpas_own(ui, key, listing_key, 0, 0);
    fpas_on_forget(ui, key, fpas_file_browser_forget);
    fpas_file_listing_source listing = { source ? source : cui_fs_list, user, current.maximum };
    const cui_file_browser_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .filter = filter,
        .folders_only = (flags & 2) != 0, .hidden = current.hidden, .no_places = (flags & 8) != 0,
        .places = place_count ? places : NULL, .place_count = place_count,
        .list = fpas_file_list_limited, .list_user = &listing, .max_entries = maximum };
    fpas_file_clear_hidden_selection(previous, filter, options.folders_only);
    const unsigned navigation = fpas_file_mask_keys(ui, enabled);
    cui_push_disabled(ui, !enabled);
    const cui_file_browser_result result = cui_file_browser_ex(ui, id, folder, (size_t)capacity, &options);
    cui_pop_disabled(ui);
    fpas_file_restore_keys(ui, navigation);
    cui__fb_state *state = cui_state(ui, key, sizeof *state);
    const bool filtered = fpas_file_clear_hidden_selection(state, filter, options.folders_only);
    if (filtered) cui_request_redraw(ui);
    *selected = filtered ? NULL : result.selected;
    *picked = enabled && !filtered ? result.picked : NULL;
    parts[0] = fpas_find(ui, list);
    cui_push_id(ui, id);
    parts[1] = fpas_find(ui, cui__hash_str(CUI_ICON_ARROW_LEFT, cui_make_id(ui, "Back")));
    parts[2] = fpas_find(ui, cui__hash_str(CUI_ICON_ARROW_RIGHT, cui_make_id(ui, "Forward")));
    parts[3] = fpas_find(ui, cui__hash_str(CUI_ICON_ARROW_UP, cui_make_id(ui, "Up")));
    parts[4] = fpas_find(ui, cui_make_id(ui, "path"));
    parts[5] = fpas_find(ui, cui_make_id(ui, "filter"));
    parts[6] = fpas_find(ui, cui__hash_str(CUI_ICON_REFRESH_CW, cui_make_id(ui, "Reload (F5)")));
    cui_pop_id(ui);
    fpas_creative_node(ui, root, key);
    return (result.navigated ? 1 : 0) | (result.error ? 2 : 0);
}
