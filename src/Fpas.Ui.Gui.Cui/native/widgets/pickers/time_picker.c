#include "../../fpas_cui.h"

int fpas_cui_time_picker(cui_ctx *ui, const char *id, int value, const char *label,
    float width, int twelve_hour, int step, int enabled, int focus, int *nodes) {
    const cui_id key = cui_make_id(ui, id), popup_key = key ^ 0x7177u;
    const cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;

    cui_push_id_int(ui, (int)popup_key);
    const cui_id hours = cui_make_id(ui, "#h"), minutes = cui_make_id(ui, "#m");
    cui_pop_id(ui);
    fpas_own(ui, key, hours, minutes, popup_key);

    int hour = value / 60, minute = value % 60;
    const cui_time_opts options = { .label = label, .width = width > 0 ? CUI_PX(width) : CUI_FIT,
        .twelve_hour = twelve_hour != 0, .minute_step = step };
    cui_push_disabled(ui, !enabled);
    cui_time_picker_ex(ui, id, &hour, &minute, &options);
    cui_pop_disabled(ui);

    const int field = fpas_find(ui, key);
    const int root = label ? ui->nodes[field].parent : field;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root;
    ui->last_id = key;

    popup = cui__popup_get(ui, popup_key, false);
    const bool open = allowed && popup && popup->open;
    if (allowed && escape && !open) ui->focus = key;
    if (nodes) {
        nodes[0] = field;
        for (int i = 1; i < 87; ++i) nodes[i] = -1;
        if (open) {
            for (int i = 0; i < (twelve_hour ? 12 : 24); ++i)
                nodes[1 + i] = fpas_find(ui, cui__hash(&i, sizeof i, hours));
            for (int i = 0; i < 60; i += step)
                nodes[25 + i] = fpas_find(ui, cui__hash(&i, sizeof i, minutes));
            if (twelve_hour)
                for (int i = 0; i < 2; ++i)
                    nodes[85 + i] = fpas_find(ui, cui__hash(&i, sizeof i, key ^ 0xA9u));
        }
    }
    return allowed ? hour * 60 + minute : value;
}
