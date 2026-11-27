// [Documentation](../../../../../docs/ui/gui/capture/dropzone.md)
#include "../../fpas_cui.h"

typedef struct {
    int drop_count;
    bool drag_hover;
} fpas_file_drag_state;

/* CUI-046: use CUI's painted hit geometry for desktop drag input.
   See docs/future/cui-upstream-bugs.md#cui-046. */
static fpas_file_drag_state fpas_filter_file_drag(cui_ctx *ui, cui_id key) {
    fpas_file_drag_state hidden = {0};
    if (ui->drop_count && !ui->drop_claimed && !fpas_interactive_at(ui, key, ui->drop_x, ui->drop_y)) {
        hidden.drop_count = ui->drop_count;
        ui->drop_count = 0;
    }
    if (ui->drag_hover && !fpas_interactive_at(ui, key, ui->drag_x, ui->drag_y)) {
        hidden.drag_hover = true;
        ui->drag_hover = false;
    }
    return hidden;
}

static void fpas_restore_file_drag(cui_ctx *ui, fpas_file_drag_state hidden) {
    if (hidden.drop_count) ui->drop_count = hidden.drop_count;
    if (hidden.drag_hover) ui->drag_hover = true;
}

int fpas_cui_dropzone(cui_ctx *ui, const char *id, const char *title, const char *description,
    const char *accept, const char *icon, float height, int width_kind, float width, int compact,
    int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui_dropzone_opts options = {
        .title = title ? cui__frame_strdup(ui, title, -1) : NULL,
        .description = description ? cui__frame_strdup(ui, description, -1) : NULL,
        .accept = accept, .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL,
        .height = height, .width = fpas_extent(width_kind, width), .compact = compact != 0
    };
    cui_push_disabled(ui, !enabled);
    const fpas_file_drag_state hidden = fpas_filter_file_drag(ui, key);
    const int event = cui_dropzone_ex(ui, id, &options);
    fpas_restore_file_drag(ui, hidden);
    cui_pop_disabled(ui);
    *node = ui->last_node = fpas_find(ui, key);
    fpas_own(ui, key, key ^ 0x1C0u, 0, 0);
    return allowed ? event : 0;
}
