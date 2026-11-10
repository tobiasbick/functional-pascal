#include "../fpas_cui.h"

int fpas_cui_button(cui_ctx *ui, const char *id, const char *text,
                    int variant, int size, int enabled, int loading, int selected, int focus,
                    const char *icon, const char *icon_right, const char *tooltip, const char *shortcut,
                    int fill, float width, unsigned int rgba) {
    cui_id key = cui_make_id(ui, id);
    if (!enabled && ui->focus == key) ui->focus = 0;
    if (enabled && focus) ui->focus_next_request = key;
    const cui_button_opts options = {
        .variant = fpas_button_variant(variant), .size = (cui_control_size)size,
        .disabled = !enabled, .loading = loading != 0, .selected = selected || variant == 3,
        .icon = icon, .icon_right = icon_right, .tooltip = tooltip, .shortcut = shortcut,
        .full_width = fill != 0, .width = width < 0 ? CUI_FIT : CUI_PX(width),
        .color = { (uint8_t)rgba, (uint8_t)(rgba >> 8), (uint8_t)(rgba >> 16), (uint8_t)(rgba >> 24) }
    };
    /* Reserve the native label slot between icons, then replace its text verbatim. */
    bool activated = cui_button_ex(ui, *text ? cui__frame_printf(ui, "x%s", id) : id, &options);
    if (*text) fpas_literal(ui, ui->last_node, text, CUI_C(ui, FOREGROUND));
    return activated ? 1 : 0;
}
