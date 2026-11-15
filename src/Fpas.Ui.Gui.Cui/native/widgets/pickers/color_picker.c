#include "../../fpas_cui.h"

uint32_t fpas_cui_color_picker(cui_ctx *ui, const char *id, uint32_t rgba,
    int enabled, int focus, int *nodes, int *open) {
    const cui_id key = cui_make_id(ui, id), popup_key = key ^ 0xC010Eu;
    const cui_color value = { (uint8_t)rgba, (uint8_t)(rgba >> 8), (uint8_t)(rgba >> 16), (uint8_t)(rgba >> 24) };
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui__popup_is_topmost(ui, popup) && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;

    fpas_own(ui, key, key ^ 1u, 0, popup_key);
    cui_color proposed = value;
    cui_push_disabled(ui, !enabled);
    cui_color_picker(ui, id, &proposed);
    cui_pop_disabled(ui);
    if (!allowed) proposed = value;

    const int trigger = fpas_find(ui, key);
    ui->last_node = trigger;
    ui->last_id = key;
    popup = cui__popup_get(ui, popup_key, false);
    *open = allowed && popup && popup->open;
    if (allowed && escape && !*open) ui->focus = key;
    if (nodes) {
        nodes[0] = trigger;
        if (*open) {
            cui_push_id_int(ui, (int)popup_key);
            nodes[1] = fpas_find(ui, cui_make_id(ui, "#sv"));
            nodes[2] = fpas_find(ui, cui_make_id(ui, "#hue"));
            for (int k = 0; k < 8; ++k)
                nodes[3 + k] = fpas_find(ui, cui__hash(&k, sizeof k, popup_key));
            cui_pop_id(ui);
        }
    }
    return (uint32_t)proposed.r | ((uint32_t)proposed.g << 8) |
        ((uint32_t)proposed.b << 16) | ((uint32_t)proposed.a << 24);
}
