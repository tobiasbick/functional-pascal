#include "../fpas_cui.h"

int fpas_cui_chip(cui_ctx *ui, const char *id, const char *label, const char *icon,
    int selected, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id) ^ 0xC41B0004u;
    if (enabled && !cui_is_disabled(ui) && focus) ui->focus_next_request = key;
    bool value = selected != 0;
    const cui_chip_opts options = { .icon = icon, .disabled = !enabled };
    const bool changed = cui_chip_ex(ui, cui__frame_printf(ui, "x%s", id), &value, &options);
    fpas_literal(ui, fpas_find(ui, key), label, CUI_C(ui, FOREGROUND));
    return changed;
}
