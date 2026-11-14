#include "../../fpas_cui.h"

typedef struct { cui_color expected; bool init; } fpas_color_selection;

uint32_t fpas_cui_color_picker(cui_ctx *ui, const char *id, uint32_t rgba,
    int enabled, int focus, int *nodes, int *open) {
    const cui_id key = cui_make_id(ui, id), popup_key = key ^ 0xC010Eu, model_key = key ^ 0x434F4C52u;
    const cui_color value = { (uint8_t)rgba, (uint8_t)(rgba >> 8), (uint8_t)(rgba >> 16), (uint8_t)(rgba >> 24) };
    const bool allowed = enabled && !cui_is_disabled(ui);
    cui_popup *popup = cui__popup_get(ui, popup_key, false);
    const bool was_open = popup && popup->open;
    const bool escape = was_open && cui__popup_is_topmost(ui, popup) && cui_key_pressed(ui, CUI_KEY_ESCAPE);
    if (!allowed && was_open) cui__popup_set(ui, popup_key, false);
    if (allowed && focus) ui->focus_next_request = key;

    fpas_own(ui, key, key ^ 1u, model_key, popup_key);
    fpas_color_selection *model = cui_state(ui, model_key, sizeof *model);
    /* CUI-013 (docs/future/cui-upstream-bugs.md#cui-013): reconcile the HSV
       state with model edits or rejected proposals while the popup is open. */
    if (model->init && (value.r != model->expected.r || value.g != model->expected.g ||
                       value.b != model->expected.b)) {
        cui__color_state *state = cui_state(ui, key, sizeof *state);
        cui__rgb_to_hsv(value, &state->h, &state->s, &state->v);
        state->init = true;
    }

    cui_color proposed = value;
    cui_push_disabled(ui, !enabled);
    cui_color_picker(ui, id, &proposed);
    cui_pop_disabled(ui);
    if (!allowed) proposed = value;
    proposed.a = value.a;
    model->expected = proposed;
    model->init = true;

    const int trigger = fpas_find(ui, key);
    ui->last_node = trigger;
    ui->last_id = key;
    popup = cui__popup_get(ui, popup_key, false);
    *open = allowed && popup && popup->open;
    if (allowed && escape && !*open) ui->focus = key;
    /* CUI-012 (docs/future/cui-upstream-bugs.md#cui-012): show keyboard focus. */
    if (allowed && ui->focus == key && ui->focus_visible) cui__focus_ring(ui, trigger, 1.0f);
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
