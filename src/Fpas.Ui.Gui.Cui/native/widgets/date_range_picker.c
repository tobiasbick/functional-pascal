#include "../fpas_cui.h"

int fpas_cui_range_picker(cui_ctx *ui, const char *id, int *start, int *end,
    const char *label, const char *placeholder, float width, int presets, int enabled, int focus,
    int *trigger, int *parts, int *nodes, int *count, int *open) {
    static const char *names[] = {
        "Today", "Yesterday", "Last 7 days", "Last 30 days", "This month", "Last month", "This year"
    };
    const cui_id key = cui_make_id(ui, id), popup_key = key ^ 0xDA7Fu;
    const cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;
    cui_date_range range = { fpas_date_from_number(*start), fpas_date_from_number(*end) };
    cui_push_id_int(ui, (int)popup_key);
    const cui_id calendar = cui_make_id(ui, "#rcal");
    const int month = fpas_range_prepare(ui, calendar, range);
    cui_pop_id(ui);
    fpas_own(ui, key, calendar, 0, popup_key);

    const cui_date_range_opts options = {
        .label = label, .placeholder = placeholder, .width = width > 0 ? CUI_PX(width) : CUI_FIT,
        .no_presets = !presets
    };
    const int first = ui->node_count;
    cui_push_disabled(ui, !allowed);
    const bool changed = cui_date_range_picker_ex(ui, id, &range, &options);
    cui_pop_disabled(ui);
    *trigger = fpas_find(ui, key);
    const int root = label ? ui->nodes[*trigger].parent : *trigger;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;
    popup = cui__popup_get(ui, popup_key, false);
    *open = allowed && popup && popup->open;
    *count = 0;
    if (*open) {
        cui_push_id_int(ui, (int)popup_key);
        fpas_range_parts(ui, "#rcal", calendar, first, month, parts, nodes, count);
        if (presets)
            for (int i = 0; i < 7; ++i) {
                cui_push_id_int(ui, i);
                fpas_range_part(ui, cui_make_id(ui, names[i]), first, -i - 1, parts, nodes, count);
                cui_pop_id(ui);
            }
        cui_pop_id(ui);
    }
    if (allowed && was_open && !*open && (escape || changed)) ui->focus = key;
    return allowed && changed && fpas_range_propose(range, start, end);
}
