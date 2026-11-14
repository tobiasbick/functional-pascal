#include "../fpas_cui.h"
#include "chrome_input.h"

void fpas_cui_dockspace_begin(cui_ctx *ui, const char *id, const char *layout, int shortcuts,
    int width_kind, float width, int height_kind, float height, float minimum_width, float minimum_height,
    int enabled) {
    const bool allowed = enabled && !cui_is_disabled(ui);
    const cui_dockspace_opts opts = { .layout = layout, .shortcuts = allowed && shortcuts,
        .width = fpas_extent(width_kind, width), .height = fpas_extent(height_kind, height),
        .min_panel_width = minimum_width, .min_panel_height = minimum_height };
    fpas_chrome_input input;
    if (!allowed) {
        /* Reject a drag that lost its input scope instead of interpreting it as a drop. */
        cui__dk_state_of(ui, id)->drag = DK_IDLE;
        fpas_chrome_pause(ui, &input);
    }
    cui_push_disabled(ui, !allowed);
    cui_dockspace_begin_ex(ui, id, &opts);
    cui_pop_disabled(ui);
    if (!allowed) fpas_chrome_resume(ui, &input);
}

int fpas_cui_dock_panel_begin(cui_ctx *ui, const char *title, const char *icon,
    int dirty, int closable, int *open, int *node) {
    ui->last_id = 0;
    const cui_dock_panel_opts opts = { .icon = icon, .dirty = dirty != 0 };
    bool shown = *open != 0;
    const bool draw = cui_dock_panel_begin_ex(ui, title, closable ? &shown : NULL, &opts);
    *open = shown;
    *node = ui->last_id ? fpas_find(ui, ui->last_id) : -1;
    return draw;
}

void fpas_cui_dock_panel_end(cui_ctx *ui) { cui_dock_panel_end(ui); }
void fpas_cui_dockspace_end(cui_ctx *ui) { cui_dockspace_end(ui); }

int fpas_cui_dockspace_load(cui_ctx *ui, const char *id, const char *layout) {
    return cui_dock_load(ui, id, layout);
}

size_t fpas_cui_dockspace_save(cui_ctx *ui, const char *id, char *buffer, size_t capacity) {
    return cui_dock_save(ui, id, buffer, capacity);
}
