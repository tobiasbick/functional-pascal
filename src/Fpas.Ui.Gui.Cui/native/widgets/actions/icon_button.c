#include "../../fpas_cui.h"

int fpas_cui_icon_button(cui_ctx *ui, const char *id, const char *icon, const char *tooltip,
                         const char *shortcut, int variant, int size, int enabled,
                         int loading, int selected, int focus) {
    if (enabled && focus) ui->focus_next_request = cui_make_id(ui, id);
    cui_button_variant style = fpas_button_variant(variant);
    if (style == CUI_BUTTON_DEFAULT) style = CUI_BUTTON_GHOST;
    const cui_button_opts options = {
        .variant = style, .size = (cui_control_size)size,
        .disabled = !enabled, .loading = loading != 0, .selected = selected || variant == 3,
        .tooltip = tooltip, .shortcut = shortcut
    };
    return cui__icon_button_id(ui, cui_make_id(ui, id), icon, &options) ? 1 : 0;
}
