// [Documentation](../../../../../docs/ui/gui/social-and-media/image-viewer.md)
#include "../../fpas_cui.h"

/** Paints a disabled viewer without sending its wheel region to hit testing.
 * CUI-068 (docs/future/cui-upstream-bugs.md#cui-068). */
static void fpas_paint_disabled_viewer(cui_ctx *ui, cui_node *node, void *data) {
    const int hits = ui->hit_count;
    cui__paint_viewer(ui, node, data);
    ui->hit_count = hits;
}

/** Releases the paint transform paired with the viewer's wheel region. */
static void fpas_retire_viewer(cui_ctx *ui, cui_id key) {
    fpas_forget_state(ui, key ^ 0x1A6E1u ^ CUI__WHEEL_XFORM);
}

/** Resolves a texture and draws the native zoom and pan controls. */
int fpas_cui_image_viewer(cui_ctx *ui, const char *id, const fpas_media_source *source,
    cui_image_view *current, int width_kind, float width, int height_kind, float height,
    float maximum, int flags, int focus) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id), texture = key ^ 0x17B41u, wheel = key ^ 0x1A6E1u;
    fpas_own(ui, key, texture, wheel, 0);
    fpas_on_forget(ui, key, fpas_retire_viewer);
    const fpas_media_image *image = fpas_media_resolve(ui, texture, source);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui) && image->texture;
    if (enabled && focus) ui->focus_next_request = key;
    const cui_image_viewer_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .max_zoom = maximum,
        .no_toolbar = (flags & 2) != 0, .checkerboard = (flags & 4) != 0 };
    const cui_id scroll_target = ui->scroll_target;
    if (!enabled && scroll_target == wheel) ui->scroll_target = 0;
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_image_viewer_ex(ui, id, image->texture, image->width, image->height, current, &options);
    cui_pop_disabled(ui);
    if (!enabled) {
        ui->scroll_target = scroll_target;
        ui->nodes[first].paint = fpas_paint_disabled_viewer;
    }
    fpas_creative_node(ui, first, key);
    return enabled && changed;
}
