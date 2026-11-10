#include "../fpas_cui.h"

typedef struct { int64_t expires; } fpas_copy_feedback;

int fpas_cui_copy_button(cui_ctx *ui, const char *id, const char *text, int enabled, int focus, int64_t now) {
    const cui_id key = cui_make_id(ui, id);
    const bool available = ui->platform.set_clipboard != NULL;
    const bool allowed = enabled && available;
    if (!allowed && ui->focus == key) ui->focus = 0;
    if (allowed && focus) ui->focus_next_request = key;

    const cui_id feedback = key ^ 0xC0B7u;
    fpas_copy_feedback *state = cui__state_lazy(ui, feedback, sizeof *state, false);
    const int64_t remaining = state ? state->expires - now : 0;
    const bool recent = remaining > 0;
    const cui_button_opts options = {
        .variant = CUI_BUTTON_GHOST, .disabled = !allowed,
        .icon = recent ? CUI_ICON_CHECK : CUI_ICON_COPY,
        .tooltip = recent ? "Copied" : "Copy",
        .color = recent ? CUI_C(ui, SUCCESS) : CUI_TRANSPARENT
    };
    const bool clicked = fpas_icon_button_face(ui, id, options);
    if (clicked) {
        ui->platform.set_clipboard(ui->platform.user, text);
        state = cui_state(ui, feedback, sizeof *state);
        state->expires = now + 1600;
        cui__wakeup_after(ui, 0.01f);
    } else if (recent) {
        cui__wakeup_after(ui, (float)remaining / 1000.0f + 0.01f);
    }
    return (clicked ? 1 : 0) | (recent ? 2 : 0) | (available ? 4 : 0);
}
