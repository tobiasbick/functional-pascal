#include "../../fpas_cui.h"

int fpas_cui_confirm_button(cui_ctx *ui, const char *id, const char *label,
    const char *confirm_label, const char *icon, int timeout_ms, int variant, int size, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id);
    cui__confirm_state *state = cui_state(ui, key ^ 0xC0F1u, sizeof *state);
    if (!enabled) {
        state->armed = false;
    }
    if (enabled && focus) ui->focus_next_request = key;

    /* Let CUI own interaction, cancellation, countdown and both animated faces.
     * Slots keep its label-ID syntax out of literal FPAS captions. */
    const fpas_morph_width saved = fpas_morph_begin(ui, &state->m,
        state->armed ? "y" : "x", state->armed ? CUI_ICON_TRIANGLE_ALERT : icon, (cui_control_size)size);
    const cui_confirm_opts options = {
        .confirm_label = "y", .icon = icon, .variant = fpas_button_variant(variant),
        .size = (cui_control_size)size, .timeout = (float)timeout_ms / 1000.0f
    };
    cui_push_disabled(ui, !enabled);
    const bool confirmed = cui_confirm_button_ex(ui, cui__frame_printf(ui, "x%s", id), &options);
    cui_pop_disabled(ui);
    const char *captions[] = { label, confirm_label };
    fpas_morph_captions(ui, "xy", captions);
    fpas_morph_end(ui, &state->m, saved, state->armed ? confirm_label : label,
        state->armed ? CUI_ICON_TRIANGLE_ALERT : icon, options.size);
    fpas_own(ui, key, key ^ 0xC0F1u, 0, 0);
    return (enabled && confirmed ? 1 : 0) | (state->armed ? 2 : 0);
}
