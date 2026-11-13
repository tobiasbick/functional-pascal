#include "../fpas_cui.h"

int fpas_cui_image_cropper(cui_ctx *ui, const char *id, const fpas_media_source *source,
    cui_rect *current, int width_kind, float width, int height_kind, float height,
    float aspect, float minimum, int flags, int focus) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id), texture = key ^ 0x17B41u;
    fpas_own(ui, key, texture, 0, 0);
    const fpas_media_image *image = fpas_media_resolve(ui, texture, source);
    const bool enabled = (flags & 1) && !cui_is_disabled(ui) && image->texture;
    if (enabled && focus) ui->focus_next_request = key;
    const cui_cropper_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .aspect = aspect,
        .min_size = minimum, .round = (flags & 2) != 0 };
    cui_push_disabled(ui, !enabled);
    const bool changed = cui_cropper_ex(ui, id, image->texture, image->width, image->height, current, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, first, key);
    return enabled && changed;
}
