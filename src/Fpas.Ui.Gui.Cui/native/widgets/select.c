#include "../fpas_cui.h"

int fpas_cui_select(cui_ctx *ui, const char *id, const char *const *items, const char *const *icons,
    int count, int *selected, const char *label, const char *placeholder,
    int searchable, int enabled, int focus, int *nodes) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id popup_id = key ^ 0x5E1Eu;
    cui_push_id_int(ui, (int)popup_id);
    const cui_id search_id = cui_make_id(ui, "#q");
    const cui_id list_id = cui_make_id(ui, "#list");
    cui_pop_id(ui);
    fpas_own(ui, key, popup_id ^ 0x4E4156u, search_id, popup_id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_popup *popup = cui__popup_get(ui, popup_id, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    if (!allowed && was_open) cui__popup_set(ui, popup_id, false);
    if (allowed && focus) ui->focus_next_request = key;

    const char **glyphs = cui__frame_alloc(ui, sizeof *glyphs * (size_t)count);
    for (int i = 0; i < count; ++i) {
        glyphs[i] = icons[i][0] ? icons[i] : NULL;
        nodes[i + 1] = -1;
    }
    const cui_select_opts options = { .label = label, .placeholder = placeholder, .icons = glyphs,
        .searchable = searchable != 0, .disabled = !allowed };
    const int first_node = ui->node_count;
    const int original = *selected;
    cui_select_ex(ui, id, items, count, selected, &options);
    if (!allowed) *selected = original;

    const int trigger = fpas_find(ui, key);
    const int root = label ? ui->nodes[trigger].parent : trigger;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;
    nodes[0] = trigger;

    popup = cui__popup_get(ui, popup_id, false);
    const bool open = allowed && popup && popup->open;
    if (allowed && was_open && !open && !ui->focus &&
        (escape || (ui->hot_layer == popup_id && ui->mreleased[CUI_MOUSE_LEFT]))) ui->focus = key;
    const int list = fpas_find(ui, list_id);
    if (list >= first_node) {
        const cui__select_state *state = cui_state(ui, key, sizeof *state);
        int child = ui->nodes[list].first;
        for (int i = 0; i < count && child >= 0; ++i) {
            if (searchable && !cui__contains_ci(items[i], state->query)) continue;
            if (strstr(items[i], "##")) {
                const int text = fpas_label_node(ui, child);
                if (text >= 0) fpas_text(ui, text, items[i]);
            }
            if (open) nodes[i + 1] = child;
            child = ui->nodes[child].next;
        }
    }
    return open;
}
