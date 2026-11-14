#include "../../fpas_cui.h"

static bool fpas_money_fits(double value, int decimals) {
    if (!isfinite(value)) return false;
    char text[64];
    const int length = snprintf(text, sizeof text, "%.*f", decimals, value);
    if (length < 0 || length >= (int)sizeof text) return false;
    const char *dot = strchr(text, '.');
    const int integers = (dot ? (int)(dot - text) : length) - (value < 0 ? 1 : 0);
    return length + (integers - 1) / 3 < (int)sizeof text;
}

void fpas_cui_money_field(cui_ctx *ui, const char *id, double *value,
    const char *label, const char *symbol, const char *code, const char *hint, int decimals,
    int width_kind, float width, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id), state_key = key ^ 0x3077u;
    cui__money_state *state = cui_state(ui, state_key, sizeof *state);
    fpas_own(ui, key, state_key, 0, 0);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (!allowed) {
        if (ui->focus == key) ui->focus = 0;
        state->had_focus = state->init = false;
    }
    if (allowed && focus) { ui->focus = key; ui->focus_visible = true; }
    if (state->init && state->last != *value) {
        if (ui->focus == key) snprintf(state->text, sizeof state->text, "%.*f", decimals, *value);
        else state->had_focus = false;
        cui_edit_state *edit = cui__state_lazy(ui, key, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, (int)strlen(state->text));
    }
    const cui_money_opts options = {
        .label = label ? cui__frame_strdup(ui, label, -1) : NULL,
        .symbol = cui__frame_strdup(ui, symbol, -1),
        .code = code ? cui__frame_strdup(ui, code, -1) : NULL,
        .hint = hint ? cui__frame_strdup(ui, hint, -1) : NULL,
        .decimals = decimals ? decimals : -1, .width = fpas_extent(width_kind, width) };
    const double original = *value;
    cui_push_disabled(ui, !allowed);
    cui_money_input_ex(ui, id, value, &options);
    cui_pop_disabled(ui);
    /* CUI-018 (docs/future/cui-upstream-bugs.md#cui-018): a typed amount can
       exceed the fixed grouped display and later be silently truncated. */
    if (!fpas_money_fits(*value, decimals)) {
        *value = original; state->last = original;
        if (ui->focus == key) snprintf(state->text, sizeof state->text, "%.*f", decimals, original);
        else cui__format_money(state->text, sizeof state->text, original, decimals);
        fpas_text_refresh(ui, key, state->text);
    }
    *node = fpas_find(ui, key);
    const int root = label || hint ? ui->nodes[*node].parent : *node;
    if (label && strstr(label, "##")) fpas_text(ui, ui->nodes[root].first, label);
    ui->last_node = root; ui->last_id = key;
}
