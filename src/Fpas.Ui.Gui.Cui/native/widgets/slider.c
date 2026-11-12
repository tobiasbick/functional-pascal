#include "../fpas_cui.h"

/* Track a pointer gesture when choosing the single-value slider's step. */
static bool fpas_slider_dragging(cui_ctx *ui, cui_id track) {
    return ui->mdown[CUI_MOUSE_LEFT] && cui__prev_rect(ui, track).w > 18 &&
        (ui->active == track || (!ui->active && ui->hot == track && ui->mpressed[CUI_MOUSE_LEFT]));
}

/* Keep layout on the complete slider while input and inspection use its track. */
static int fpas_finish_slider(cui_ctx *ui, cui_id group, cui_id track, const char *label, bool enabled) {
    const int column = fpas_find(ui, group);
    fpas_literal(ui, column, label, CUI_C(ui, FOREGROUND));
    if (!enabled) ui->nodes[column].opacity = CUI__DISABLED;
    fpas_own(ui, track, group, 0, 0);
    ui->last_id = track;
    return fpas_find(ui, track);
}

int fpas_cui_slider(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, int show_value, int decimal_places, int percent,
    int enabled, int focus, int *track) {
    const cui_id group = cui_make_id(ui, id);
    const cui_id key = group ^ 0x51u;
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const bool keyboard = (ui->focus == key || focus) &&
        (cui_key_pressed(ui, CUI_KEY_LEFT) || cui_key_pressed(ui, CUI_KEY_RIGHT) ||
         cui_key_pressed(ui, CUI_KEY_UP) || cui_key_pressed(ui, CUI_KEY_DOWN) ||
         cui_key_pressed(ui, CUI_KEY_HOME) || cui_key_pressed(ui, CUI_KEY_END));
    const bool adjusting = allowed && (keyboard || fpas_slider_dragging(ui, key));

    /* Construct the numeric format here so CUI receives exactly one conversion. */
    char format[16];
    snprintf(format, sizeof format, "%%.%df%s", decimal_places, percent ? "%%" : "");
    const cui_slider_opts options = {
        .step = adjusting ? step : 0, .show_value = show_value != 0,
        .format = format, .disabled = !enabled
    };
    float next = *value;
    cui_slider_ex(ui, cui__frame_printf(ui, "x%s", id), &next, minimum, maximum, &options);
    *track = fpas_finish_slider(ui, group, key, label, allowed);
    if (!adjusting || !isfinite(next)) return 0;
    next = cui__clampf(next, minimum, maximum);
    const bool changed = next != *value;
    *value = next;
    return changed;
}
