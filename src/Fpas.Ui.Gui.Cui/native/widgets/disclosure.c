#include "../fpas_cui.h"

int fpas_cui_disclosure_begin(cui_ctx *ui, const char *id, const char *title,
    int kind, int open, int enabled, int focus, const char *description,
    const char *icon, const char *badge, int tone, int *node, int *header) {
    const cui_id key = cui_make_id(ui, id) ^ (kind ? 0xEC4Du : 0xACC00006u);
    /* Reserve the temporary header scope and CUI's outgoing-body scope. */
    if (ui->disabled_n > 30) { *node = *header = -1; return -1; }
    if (enabled && !cui_is_disabled(ui) && focus) ui->focus_next_request = key;
    bool proposed = open != 0;
    const int first = ui->node_count;
    const char *caption = cui__frame_printf(ui, "x%s", id);
    const cui_expandable_card_opts options = {
        .description = description && *description ? description : NULL,
        .icon = icon && *icon ? icon : NULL, .badge = badge && *badge ? badge : NULL,
        .badge_tone = (cui_tone)tone
    };
    cui_push_disabled(ui, !enabled);
    const bool draw = kind ? cui_expandable_card_begin(ui, caption, &proposed, &options)
                           : cui_accordion_begin(ui, caption, &proposed);
    /* Only the header inherits host input admission. A nested modal admits its own body.
       Preserve the outgoing-body scope that CUI itself owns until end(). */
    if (draw && !proposed) cui_pop_disabled(ui);
    cui_pop_disabled(ui);
    if (draw && !proposed) cui_push_disabled(ui, true);
    *node = first;
    *header = fpas_find(ui, key);
    fpas_literal(ui, *header, title, CUI_C(ui, FOREGROUND));
    ui->last_node = *node;
    ui->last_id = ui->nodes[*node].id;
    fpas_own(ui, key, key ^ 0xC0u, 0, 0);
    return (draw ? 1 : 0) | (proposed != (open != 0) ? 2 : 0) | (proposed ? 4 : 0);
}

void fpas_cui_disclosure_end(cui_ctx *ui, int kind, int node) {
    if (kind) cui_expandable_card_end(ui); else cui_accordion_end(ui);
    ui->last_node = node;
    ui->last_id = ui->nodes[node].id;
}

int fpas_cui_expandable_text(cui_ctx *ui, const char *id, const char *text,
    int lines, float size, int colors, unsigned int color, unsigned int fade_to,
    int enabled, int focus, int *toggle) {
    const cui_id key = cui_make_id(ui, id);
    /* CUI-006 (docs/future/cui-upstream-bugs.md#cui-006): CUI changes the link's
       identity on expansion. Preserve native behavior; do not synthesize focus. */
    const cui__et_state *before = cui__state_lazy(ui, key, sizeof *before, false);
    cui_push_id(ui, id);
    const cui_id more = cui_make_id(ui, "Show more") ^ 0x11AC0009u;
    const cui_id less = cui_make_id(ui, "Show less") ^ 0x11AC0009u;
    cui_pop_id(ui);
    if (enabled && !cui_is_disabled(ui) && focus)
        ui->focus_next_request = before && before->open ? less : more;
    const cui_expandable_text_opts options = {
        .lines = lines, .size = size,
        .color = colors & 1 ? (cui_color){color & 255, (color >> 8) & 255, (color >> 16) & 255, color >> 24} : (cui_color){0},
        .fade_to = colors & 2 ? (cui_color){fade_to & 255, (fade_to >> 8) & 255, (fade_to >> 16) & 255, fade_to >> 24} : (cui_color){0}
    };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_expandable_text_ex(ui, id, text, &options);
    cui_pop_disabled(ui);
    *toggle = fpas_find(ui, more);
    if (*toggle < 0) *toggle = fpas_find(ui, less);
    const cui__et_state *state = cui__state_lazy(ui, key, sizeof *state, false);
    ui->last_node = fpas_find(ui, key);
    ui->last_id = key;
    fpas_own(ui, key, more, less, 0);
    return (changed ? 1 : 0) | (state && state->open ? 2 : 0);
}
