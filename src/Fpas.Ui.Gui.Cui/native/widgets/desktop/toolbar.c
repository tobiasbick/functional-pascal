#include "../../fpas_cui.h"

void fpas_cui_toolbar_begin(cui_ctx *ui, const char *id, int style, int no_border, int enabled) {
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, 0, 0, key ^ 0x0F10Fu);
    const cui_toolbar_opts options = { .style = (cui_toolbar_style)style, .no_border = no_border != 0 };
    cui_push_disabled(ui, !enabled);
    cui_toolbar_begin(ui, id, &options);
}

void fpas_cui_toolbar_end(cui_ctx *ui) {
    cui__toolbar_build *bar = cui__slots(ui)->tb;
    const cui_id key = bar ? bar->id : 0;
    cui_toolbar_end(ui);
    cui_pop_disabled(ui);
    ui->last_id = key;
    ui->last_node = fpas_find(ui, key);
}

int fpas_cui_toolbar_button(cui_ctx *ui, const char *icon, const char *label, const char *tooltip,
    const char *shortcut, int toggle, int tone, int flags) {
    /* CUI's overflow menu is built at EndToolbar and borrows its icons. */
    if (icon) icon = cui__frame_strdup(ui, icon, -1);
    bool on = toggle > 0;
    const cui_tool_opts options = { .label = label, .tooltip = tooltip, .shortcut = shortcut,
        .toggle = toggle >= 0 ? &on : NULL, .tone = (cui_tone)tone, .disabled = (flags & 1) != 0,
        .flyout = (flags & 2) != 0 };
    const bool clicked = flags & 4 ? cui_toolbar_primary_ex(ui, icon, &options)
        : cui_toolbar_button_ex(ui, icon, &options);
    return clicked ? (on ? 1 : 0) : -1;
}

int fpas_cui_toolbar_radio(cui_ctx *ui, const char *id, const char *const *icons,
    const char *const *labels, int count, int value) {
    if (count <= 0) return -1;
    if (count > 32) count = 32;
    const char **copy = cui__frame_alloc(ui, (size_t)count * sizeof *copy);
    for (int i = 0; i < count; ++i)
        copy[i] = icons[i] ? cui__frame_strdup(ui, icons[i], -1) : NULL;
    const cui_toolbar_radio_opts options = { .labels = labels };
    return cui_toolbar_radio_ex(ui, id, copy, count, &value, &options) ? value : -1;
}

int fpas_cui_toolbar_cycle(cui_ctx *ui, const char *id, const char *const *labels,
    int count, int value, const char *tooltip) {
    const cui_tool_opts options = { .tooltip = tooltip };
    return cui_toolbar_cycle_ex(ui, id, labels, count, &value, &options) ? value : -1;
}
