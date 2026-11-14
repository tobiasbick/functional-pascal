#include "../../fpas_cui.h"

int fpas_cui_date_picker(cui_ctx *ui, const char *id, int value, const char *label,
    const char *placeholder, float width, int enabled, int focus,
    int *trigger, int *parts, int *nodes, int *count) {
    const cui_id key = cui_make_id(ui, id), popup_key = key ^ 0xDA7Eu;
    const cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;

    cui_date date = fpas_date_from_number(value);
    cui_push_id_int(ui, (int)popup_key);
    const cui_id calendar = cui_make_id(ui, "#cal");
    cui_pop_id(ui);
    fpas_own(ui, key, calendar, 0, popup_key);

    const cui_date_opts options = {
        .label = label, .placeholder = placeholder, .width = width > 0 ? CUI_PX(width) : CUI_FIT
    };
    const int first = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_date_picker_ex(ui, id, &date, &options);
    cui_pop_disabled(ui);

    *trigger = fpas_find(ui, key);
    const int root = label ? ui->nodes[*trigger].parent : *trigger;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;
    popup = cui__popup_get(ui, popup_key, false);
    const bool open = allowed && popup && popup->open;
    if (allowed && was_open && !open && (escape || changed)) ui->focus = key;
    /* CUI-012 (docs/future/cui-upstream-bugs.md#cui-012): the date field
       accepts Tab focus but omits the native focus ring. */
    if (allowed && ui->focus == key && ui->focus_visible) cui__focus_ring(ui, *trigger, 1.0f);

    *count = 0;
    if (open && parts) {
        fpas_calendar_parts(ui, calendar, first, parts, nodes, count);
    }
    const int proposed = fpas_date_number(date);
    return allowed && changed && proposed >= 0 ? proposed : value;
}
