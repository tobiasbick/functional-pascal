#include "../fpas_cui.h"

/* Match cui_progress_ex's positional ID and cui_animate_spring's state salt. */
static cui_id fpas_progress_state(cui_ctx *ui, const char *label, int node) {
    const cui_id id = cui_make_id(ui, label ? label : "progress");
    return cui__hash(&node, sizeof node, id) ^ 0xA11A11u;
}

int fpas_cui_alert(cui_ctx *ui, const char *id, const char *title,
    const char *description, const char *icon, int tone, int dismissible,
    int enabled, int focus, int *node, int *dismiss, unsigned int *key) {
    const char *caption = cui__frame_printf(ui, "x%s", id);
    *key = cui_make_id(ui, caption) ^ 0xA1E70010u;
    cui_push_id(ui, caption);
    const cui_id close = cui__hash_str(CUI_ICON_X, cui_make_id(ui, "Dismiss"));
    cui_pop_id(ui);
    if (enabled && dismissible && !cui_is_disabled(ui) && focus)
        ui->focus_next_request = close;
    const cui_alert_opts options = {
        .description = *description ? description : NULL, .tone = (cui_tone)tone,
        .icon = *icon ? icon : NULL, .dismissible = dismissible != 0
    };
    const int first = ui->node_count;
    cui_push_disabled(ui, !enabled);
    const bool visible = cui_alert_ex(ui, caption, &options);
    cui_pop_disabled(ui);
    *node = ui->node_count > first ? first : -1;
    *dismiss = *node >= 0 && dismissible ? fpas_find(ui, close) : -1;
    if (*node >= 0) fpas_literal(ui, *node, title, CUI_C(ui, FOREGROUND));
    ui->last_node = *node;
    ui->last_id = *key;
    fpas_own(ui, *key, close, 0, 0);
    return (visible ? 1 : 0) | (!visible && *node >= 0 ? 2 : 0);
}

void fpas_cui_progress(cui_ctx *ui, const char *id, float value,
    int tone, int indeterminate, float height, const char *label, int show_value) {
    const cui_id key = cui_make_id(ui, id);
    const int first = ui->node_count;
    cui_push_id(ui, id);
    const char *caption = *label ? cui__frame_printf(ui, "x%s", id) : NULL;
    /* CUI keys interpolation by label and node position. Retire the old key
       if surrounding content changes position; interpolation stays native. */
    fpas_own(ui, key, fpas_progress_state(ui, caption, first), 0, 0);
    const cui_progress_opts options = {
        .tone = (cui_tone)tone, .indeterminate = indeterminate != 0,
        .height = height, .label = caption, .show_value = show_value != 0
    };
    cui_progress_ex(ui, value, &options);
    cui_pop_id(ui);
    ui->nodes[first].id = key;
    if (caption) fpas_literal(ui, first, label, CUI_C(ui, FOREGROUND));
    ui->last_node = first;
    ui->last_id = key;
}

void fpas_cui_spinner(cui_ctx *ui, const char *id, float size) {
    const cui_id key = cui_make_id(ui, id);
    cui_spinner(ui, size);
    ui->nodes[ui->last_node].id = key;
    ui->last_id = key;
}

void fpas_cui_progress_ring(cui_ctx *ui, const char *id, const char *label,
    float value, float size) {
    const cui_id key = cui_make_id(ui, id);
    /* CUI owns interpolation; retire its salted state when this identity leaves. */
    fpas_own(ui, key, key ^ 0xA11A11u, 0, 0);
    cui_progress_ring(ui, id, value, size);
    cui_node *node = &ui->nodes[ui->last_node];
    /* Keep native painting, but separate the literal caption from CUI ID syntax. */
    ((cui__ring_paint *)node->paint_data)->label = *label ? cui__frame_strdup(ui, label, -1) : NULL;
    node->id = key;
    ui->last_id = key;
}

void fpas_cui_meter(cui_ctx *ui, const char *id, const char *label,
    float used, float total, const char *unit) {
    const cui_id key = cui_make_id(ui, id);
    const char *caption = cui__frame_printf(ui, "x%s", id);
    const int first = ui->node_count;
    cui_meter(ui, caption, used, total, *unit ? unit : NULL);
    /* The native meter contains a caption row followed by its progress leaf.
       Retain that leaf's positional interpolation state with the meter. */
    const int bar = ui->nodes[ui->nodes[first].first].next;
    cui_push_id(ui, caption);
    const cui_id animation = fpas_progress_state(ui, NULL, bar);
    cui_pop_id(ui);
    fpas_own(ui, key, animation, 0, 0);
    ui->nodes[first].id = key;
    fpas_literal(ui, first, label, CUI_C(ui, FOREGROUND));
}
