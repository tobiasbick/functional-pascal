#include "../fpas_cui.h"

static void fpas_file_browser_forget(cui_ctx *ui, cui_id key) {
    const cui_id rows = cui__hash_str("rows", key);
    const cui_id path = cui__hash_str("path", key);
    fpas_forget_state(ui, rows);
    fpas_forget_state(ui, path ^ 0x9A7B0u);
    fpas_forget_state(ui, path ^ 0xED17u);
    fpas_forget_state(ui, cui__hash_str("filter", key));
}

int fpas_cui_file_browser(cui_ctx *ui, const char *id, char *folder, int capacity,
    int width_kind, float width, int height_kind, float height, const char *filter,
    const char *const *places, int place_count, cui_file_list_fn source, void *user, int maximum,
    int revision, int flags, int focus, const char **selected, const char **picked, int *parts) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), list = key ^ 0x115Du;
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    if (enabled && focus) ui->focus_next_request = list;
    const cui_id revision_key = key ^ 0xF11Eu;
    int *seen = cui_state(ui, revision_key, sizeof *seen);
    cui__fb_state *previous = cui__state_lazy(ui, key, sizeof *previous, false);
    if (previous && *seen != revision) previous->reload = true;
    *seen = revision;
    fpas_own(ui, key, revision_key, 0, 0);
    fpas_on_forget(ui, key, fpas_file_browser_forget);
    const cui_file_browser_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .filter = filter, .folders_only = (flags & 2) != 0,
        .hidden = (flags & 4) != 0, .no_places = (flags & 8) != 0,
        .places = place_count ? places : NULL, .place_count = place_count,
        .list = source, .list_user = user, .max_entries = maximum };
    const uint8_t reload = ui->kpressed[CUI_KEY_F5];
    const uint8_t left = ui->kpressed[CUI_KEY_LEFT], right = ui->kpressed[CUI_KEY_RIGHT];
    if (!enabled) ui->kpressed[CUI_KEY_F5] = ui->kpressed[CUI_KEY_LEFT] = ui->kpressed[CUI_KEY_RIGHT] = 0;
    cui_push_disabled(ui, !enabled);
    const cui_file_browser_result result = cui_file_browser_ex(ui, id, folder, (size_t)capacity, &options);
    cui_pop_disabled(ui);
    if (!enabled) {
        ui->kpressed[CUI_KEY_F5] = reload;
        ui->kpressed[CUI_KEY_LEFT] = left;
        ui->kpressed[CUI_KEY_RIGHT] = right;
    }
    *selected = result.selected;
    *picked = enabled ? result.picked : NULL;
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
