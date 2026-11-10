#include "../fpas_cui.h"

static bool fpas_icon_button_face(cui_ctx *ui, const char *id, cui_button_opts options) {
    /* Keep the native icon geometry and a stable ID across glyph and tooltip changes. */
    options.width = CUI_PX(cui__control_height(ui, options.size));
    bool activated = cui_button_ex(ui, id, &options);
    cui_node *node = &ui->nodes[ui->last_node];
    node->pad[1] = node->pad[3] = 0;
    node->self_align = CUI__ALIGN_NOSTRETCH;
    return activated;
}

int fpas_cui_icon_button(cui_ctx *ui, const char *id, const char *icon, const char *tooltip,
                         const char *shortcut, int variant, int size, int enabled,
                         int loading, int selected, int focus) {
    if (enabled && focus) ui->focus_next_request = cui_make_id(ui, id);
    cui_button_variant style = fpas_button_variant(variant);
    if (style == CUI_BUTTON_DEFAULT) style = CUI_BUTTON_GHOST;
    const cui_button_opts options = {
        .variant = style, .size = (cui_control_size)size, .icon = icon,
        .disabled = !enabled, .loading = loading != 0, .selected = selected || variant == 3,
        .tooltip = tooltip, .shortcut = shortcut
    };
    return fpas_icon_button_face(ui, id, options) ? 1 : 0;
}
