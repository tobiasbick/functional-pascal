// [Documentation](../../../../../docs/ui/gui/capture/dropzone.md)
#include "../../fpas_cui.h"

int fpas_cui_dropzone(cui_ctx *ui, const char *id, const char *title, const char *description,
    const char *accept, const char *icon, float height, int compact, int enabled, int focus, int *node) {
    const cui_id key = cui_make_id(ui, id);
    const bool allowed = enabled && !cui_is_disabled(ui);
    if (allowed && focus) ui->focus_next_request = key;
    const cui_dropzone_opts options = {
        .title = title ? cui__frame_strdup(ui, title, -1) : NULL,
        .description = description ? cui__frame_strdup(ui, description, -1) : NULL,
        .accept = accept, .icon = icon ? cui__frame_strdup(ui, icon, -1) : NULL,
        .height = height, .compact = compact != 0
    };
    cui_push_disabled(ui, !enabled);
    const int event = cui_dropzone_ex(ui, id, &options);
    cui_pop_disabled(ui);
    *node = ui->last_node = fpas_find(ui, key);
    fpas_own(ui, key, key ^ 0x1C0u, 0, 0);
    return allowed ? event : 0;
}
