#include "../../fpas_cui.h"

void fpas_cui_display_image(cui_ctx *ui, const char *id, const fpas_media_source *source, int width_kind, float width, int height_kind, float height, float radius, uint32_t tint) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id), texture = key ^ 0x1A6E2u;
    fpas_own(ui, key, texture, 0, 0);
    const fpas_media_image *image = fpas_media_resolve(ui, texture, source);
    const cui_image_opts options = { .width = fpas_extent(width_kind, width),
        .height = fpas_extent(height_kind, height), .radius = radius, .tint = fpas_chart_color(-1, tint) };
    cui_image_ex(ui, image->texture, &options);
    fpas_creative_node(ui, root, key);
}
