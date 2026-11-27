// [Documentation](../../../../../docs/ui/gui/pickers/multi-select.md)
#include "../../fpas_cui.h"

int fpas_cui_multi_select(cui_ctx *ui, const char *id, const char *const *labels,
    const char *const *icons, int count, int *selected, const char *label, const char *placeholder,
    int max_chips, int searchable, int width_kind, float width, int enabled, int focus, int *nodes, int *open) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id popup_key = key ^ 0x3517u;
    const cui_id navigation_key = popup_key ^ 0x4E4156u;
    cui_push_id_int(ui, (int)popup_key);
    const cui_id search = cui_make_id(ui, "#q");
    const cui_id list = cui_make_id(ui, "#list");
    cui_pop_id(ui);
    fpas_own(ui, key, navigation_key, search, popup_key);

    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool top = was_open && cui__popup_is_topmost(ui, popup);
    const bool escape = top && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;

    /* CUI-043 (docs/future/cui-upstream-bugs.md#cui-043): an exiting menu
       retains its paint pass, but its old highlight must not receive keys. */
    if ((!was_open || escape) && (cui_key_pressed(ui, CUI_KEY_ENTER) || cui_key_pressed(ui, CUI_KEY_SPACE))) {
        cui__menu_nav *navigation = cui__state_lazy(ui, navigation_key, sizeof *navigation, false);
        if (navigation) navigation->highlight = -1;
    }

    /* CUI-042 (docs/future/cui-upstream-bugs.md#cui-042): printable space
       belongs to the active search field, not the highlighted checkbox. */
    const uint8_t space_pressed = ui->kpressed[CUI_KEY_SPACE];
    const bool search_space = space_pressed && allowed && top && searchable &&
        (ui->focus == search || popup->opened_frame + 1 == ui->frame) &&
        memchr(ui->text_in, ' ', (size_t)ui->text_in_len);
    bool *values = cui__frame_alloc(ui, (size_t)count * sizeof *values);
    const char **glyphs = cui__frame_alloc(ui, (size_t)count * sizeof *glyphs);
    for (int i = 0; i < count; ++i) {
        values[i] = selected[i] != 0;
        glyphs[i] = icons[i][0] ? icons[i] : NULL;
        nodes[i + 1] = -1;
    }
    const cui_multi_select_opts options = { .label = label, .placeholder = placeholder,
        .icons = glyphs, .max_chips = max_chips, .searchable = searchable != 0,
        .width = fpas_extent(width_kind, width) };
    const int first_node = ui->node_count;
    if (search_space) ui->kpressed[CUI_KEY_SPACE] = 0;
    cui_push_disabled(ui, !allowed);
    const bool changed = cui_multi_select_ex(ui, id, labels, count, values, &options);
    cui_pop_disabled(ui);
    if (search_space) ui->kpressed[CUI_KEY_SPACE] = space_pressed;

    const int field = fpas_find(ui, key);
    nodes[0] = field;
    const int chips = ui->nodes[field].first;
    int chip = ui->nodes[chips].first;
    const int limit = max_chips > 0 ? max_chips : 3;
    for (int i = 0, shown = 0; i < count && shown < limit && chip >= 0; ++i) {
        if (!selected[i]) continue;
        if (strstr(labels[i], "##")) fpas_text(ui, ui->nodes[chip].first, labels[i]);
        chip = ui->nodes[chip].next;
        ++shown;
    }
    ui->last_node = label ? ui->nodes[field].parent : field;
    ui->last_id = key;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[ui->last_node].first, label);
    popup = cui__popup_get(ui, popup_key, false);
    *open = allowed && popup && popup->open;
    if ((!*open || !searchable) && ui->focus == search) {
        ui->focus = allowed ? key : 0;
        if (ui->edit == cui__state_lazy(ui, search, sizeof(cui_edit_state), false)) ui->edit = NULL;
    }
    if (allowed && escape && !*open && !ui->focus) ui->focus = key;
    const int list_node = fpas_find(ui, list);
    if (list_node >= first_node) {
        const cui__ms_state *state = cui_state(ui, key, sizeof *state);
        int row = ui->nodes[list_node].first;
        for (int i = 0; i < count && row >= 0; ++i) {
            if (searchable && !cui__contains_nocase(labels[i], state->query)) continue;
            if (strstr(labels[i], "##")) {
                const int text = fpas_label_node(ui, row);
                if (text >= 0) fpas_text(ui, text, labels[i]);
            }
            if (*open) nodes[i + 1] = row;
            row = ui->nodes[row].next;
        }
    }
    if (allowed && changed)
        for (int i = 0; i < count; ++i) selected[i] = values[i];
    return allowed && changed;
}
