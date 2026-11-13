#include "../fpas_cui.h"

int fpas_cui_result_card(cui_ctx *ui, const char *id, const fpas_media_source *source, const char *title,
    const char *caption, const char *badge, int tone, int width_kind, float width, float aspect, int enabled, int focus) {
    const cui_id key = cui_make_id(ui, id), texture = key ^ 0x52455355u;
    fpas_own(ui, key, texture, 0, 0);
    const fpas_media_image *image = fpas_media_resolve(ui, texture, source);
    const cui_result_card_opts options = { .caption = caption, .badge = badge, .badge_tone = (cui_tone)tone,
        .image_w = image->width, .image_h = image->height, .width = fpas_extent(width_kind, width), .aspect = aspect };
    if (enabled && focus) ui->focus_next_request = key;
    cui_push_disabled(ui, !enabled);
    const bool activated = cui_result_card_ex(ui, id, image->texture, title, &options);
    cui_pop_disabled(ui);
    return enabled && activated;
}
