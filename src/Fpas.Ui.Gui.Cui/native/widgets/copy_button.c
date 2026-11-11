#include "../fpas_cui.h"

int fpas_cui_copy_button(cui_ctx *ui, const char *id, const char *text, int enabled, int focus) {
    const bool available = ui->platform.set_clipboard != NULL;
    const bool allowed = enabled && available;
    const cui_id feedback = cui_make_id(ui, id) ^ 0xC0B7u;
    const cui__copy_state *state = cui_state(ui, feedback, sizeof *state);
    const bool recent = state->copied_at > 0 && ui->time - state->copied_at < 1.6;
    if (allowed && focus) {
        cui_push_id(ui, id);
        ui->focus_next_request = cui_make_id(ui, "#copy-button");
        cui_pop_id(ui);
    }
    cui_push_disabled(ui, !allowed);
    const bool clicked = cui_copy_button(ui, id, text);
    cui_pop_disabled(ui);
    return (clicked ? 1 : 0) | (recent ? 2 : 0) | (available ? 4 : 0);
}
