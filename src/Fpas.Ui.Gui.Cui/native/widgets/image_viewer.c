#include "../fpas_cui.h"

int fpas_cui_image_viewer(cui_ctx *ui, const char *id, const fpas_media_source *source,
    cui_image_view *current, int width_kind, float width, int height_kind, float height,
    float maximum, int flags, int focus) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id), texture = key ^ 0x17B41u;
    fpas_own(ui, key, texture, 0, 0);
    const fpas_media_image *image = fpas_media_resolve(ui, texture, source);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui) && image->texture;
    if (enabled && focus) ui->focus_next_request = key;
    const cui_image_viewer_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .max_zoom = maximum,
        .no_toolbar = (flags & 2) != 0, .checkerboard = (flags & 4) != 0 };
    const cui_id scroll_target = ui->scroll_target;
    if (!enabled && scroll_target == (key ^ 0x1A6E1u)) ui->scroll_target = 0;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_image_viewer_ex(ui, id, image->texture, image->width, image->height, current, &options);
    cui_pop_disabled(ui);
    if (!enabled) ui->scroll_target = scroll_target;
    fpas_creative_node(ui, first, key);
    return enabled && changed;
}
