#include "../../fpas_cui.h"

static void fpas_path_forget(cui_ctx *ui, cui_id key) {
    cui_popup *popup = cui__popup_get(ui, key ^ 0x3011Eu, false);
    if (popup) { popup->open = false; memset(&popup->s, 0, sizeof popup->s); }
}

int fpas_cui_path_bar(cui_ctx *ui, const char *id, const char *const *segments, int count,
    const char *text, const char *icon, const char *placeholder, int size, int width_kind, float width,
    int flags, int focus, const char **submitted, int *parts, int *root, uint32_t *retained_key) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui);
    const bool editable = (flags & 2) && enabled;
    if (enabled && focus >= 0 && focus < count)
        ui->focus_next_request = cui__pb_item_id(ui, id, focus, "seg");
    const cui_path_bar_opts options = { .icon = icon, .text = text, .placeholder = placeholder,
        .size = size, .width = fpas_extent(width_kind, width), .menus = (flags & 4) != 0, .no_edit = !editable };
    cui_push_disabled(ui, !enabled);
    const cui_path_bar_result result = cui_path_bar_ex(ui, id, segments, count, &options);
    cui_pop_disabled(ui);
    *submitted = enabled ? result.submitted : NULL;
    for (int i = 0; i < count; ++i) {
        parts[i * 2] = fpas_find(ui, cui__pb_item_id(ui, id, i, "seg"));
        parts[i * 2 + 1] = options.menus ? fpas_find(ui, cui__pb_item_id(ui, id, i, "sep")) : -1;
    }
    parts[count * 2] = fpas_find(ui, key ^ 0xED17u);
    *root = first;
    *retained_key = key;
    fpas_own(ui, key, key ^ 0x9A7B0u, key ^ 0xED17u, key ^ 0x5E9A0u);
    fpas_on_forget(ui, key, fpas_path_forget);
    return enabled ? result.clicked : -1;
}

int fpas_cui_path_menu_begin(cui_ctx *ui, const char *id) {
    int segment = -1;
    return cui_path_bar_menu_begin(ui, id, &segment) ? segment : -1;
}

void fpas_cui_path_menu_end(cui_ctx *ui) {
    cui_path_bar_menu_end(ui);
}

int fpas_cui_path_menu_item(cui_ctx *ui, const char *label, int index) {
    cui_push_id_int(ui, index);
    const bool picked = cui_menu_item_ex(ui, label, NULL);
    cui_pop_id(ui);
    return picked;
}
