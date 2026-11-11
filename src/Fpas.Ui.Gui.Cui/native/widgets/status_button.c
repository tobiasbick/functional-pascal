#include "../fpas_cui.h"

int fpas_cui_status_button(cui_ctx *ui, const char *id, const char *label, int status,
    const char *busy_label, const char *success_label, const char *error_label, const char *icon,
    int variant, int size, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id);
    cui__status_state *state = cui_state(ui, key ^ 0x57A7u, sizeof *state);
    if (enabled && focus) ui->focus_next_request = key;
    const char *slots[] = { "a", "b", "c", "d" };
    const char *captions[] = { label, busy_label, success_label, error_label };
    const char *icons[] = { icon, "spin", CUI_ICON_CHECK, CUI_ICON_CIRCLE_ALERT };
    const int shown = state->m.init ? (int)state->shown : status;
    const fpas_morph_width saved = fpas_morph_begin(ui, &state->m, slots[shown], icons[shown], (cui_control_size)size);
    const cui_status_button_opts options = {
        .busy_label = slots[1], .success_label = slots[2], .error_label = slots[3],
        .icon = icon, .variant = fpas_button_variant(variant), .size = (cui_control_size)size
    };
    cui_push_disabled(ui, !enabled);
    const bool activated = cui_status_button_ex(ui, cui__frame_printf(ui, "a%s", id), (cui_status)status, &options);
    cui_pop_disabled(ui);
    fpas_morph_captions(ui, "abcd", captions);
    fpas_morph_end(ui, &state->m, saved, captions[status], icons[status], options.size);
    if (!enabled) ui->nodes[ui->last_node].opacity *= CUI__DISABLED;
    fpas_own(ui, key, key ^ 0x57A7u, 0, 0);
    return activated ? 1 : 0;
}
