#include "../../fpas_cui.h"

int fpas_cui_image_compare(cui_ctx *ui, const char *id, const fpas_media_source *before,
    const fpas_media_source *after, float *position, int width_kind, float width, float height,
    float aspect, const char *before_label, const char *after_label, int enabled, int focus) {
    const int first = ui->node_count;
    const cui_id key = cui_make_id(ui, id), a = key ^ 0x17B41u, b = key ^ 0x17B42u;
    fpas_own(ui, key, a, b, 0);
    const fpas_media_image *left = fpas_media_resolve(ui, a, before);
    const fpas_media_image *right = fpas_media_resolve(ui, b, after);
    const bool accept = enabled && !cui_is_disabled(ui) && left->texture && right->texture;
    if (accept && focus) ui->focus_next_request = key;
    const cui_compare_opts options = { .width = fpas_extent(width_kind, width), .height = height,
        .aspect = aspect > 0 ? aspect : (float)left->width / CUI_MAX(left->height, 1),
        .before_label = cui__frame_strdup(ui, before_label, -1),
        .after_label = cui__frame_strdup(ui, after_label, -1) };
    cui_push_disabled(ui, !accept);
    const bool changed = cui_image_compare_ex(ui, id, left->texture, right->texture, position, &options);
    cui_pop_disabled(ui);
    fpas_creative_node(ui, first, key);
    return accept && changed;
}
