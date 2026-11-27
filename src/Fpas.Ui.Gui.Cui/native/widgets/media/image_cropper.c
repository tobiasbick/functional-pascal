// [Documentation](../../../../../docs/ui/gui/social-and-media/image-cropper.md)
#include "../../fpas_cui.h"

/** Keeps the native resize anchor when a crop side needs correction. */
static cui_vec2 fpas_crop_anchor(const cui__cr_state *state, cui_rect crop) {
    if (state->mode == CUI__CR_NEW) {
        return (cui_vec2){ state->px > crop.x + crop.w * 0.5f,
            state->py > crop.y + crop.h * 0.5f };
    }
    float x = 0.5f, y = 0.5f;
    const int mode = state->mode;
    if (mode == CUI__CR_W || mode == CUI__CR_NW || mode == CUI__CR_SW) x = 1;
    if (mode == CUI__CR_E || mode == CUI__CR_NE || mode == CUI__CR_SE) x = 0;
    if (mode == CUI__CR_N || mode == CUI__CR_NE || mode == CUI__CR_NW) y = 1;
    if (mode == CUI__CR_S || mode == CUI__CR_SE || mode == CUI__CR_SW) y = 0;
    return (cui_vec2){ x, y };
}

/** Reconciles the crop ratio and minimum after CUI clamps the two sides independently.
 * CUI-069 (docs/future/cui-upstream-bugs.md#cui-069). */
static void fpas_constrain_crop(cui_rect *crop, const cui__cr_state *state, double aspect,
    float minimum, int image_width, int image_height) {
    const double maximum = fmin(image_width, image_height * aspect);
    const double required = fmin(maximum, (minimum > 0 ? minimum : 16) * fmax(1, aspect));
    const double width = fmin(maximum, fmax(required, fmax(crop->w, crop->h * aspect)));
    const double height = width / aspect;
    if (fabs(width - crop->w) <= width * 1e-6 && fabs(height - crop->h) <= height * 1e-6) return;
    const cui_vec2 anchor = fpas_crop_anchor(state, *crop);
    const float w = (float)width, h = (float)height;
    crop->x = cui__clampf(crop->x + (crop->w - w) * anchor.x, 0, image_width - w);
    crop->y = cui__clampf(crop->y + (crop->h - h) * anchor.y, 0, image_height - h);
    crop->w = w;
    crop->h = h;
}

/** Draws the native cropper and proposes the final image rectangle. */
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
    const cui_rect previous = *current;
    cui_push_disabled(ui, !enabled);
    cui_cropper_ex(ui, id, image->texture, image->width, image->height, current, &options);
    cui_pop_disabled(ui);
    const float ratio = options.round ? 1 : options.aspect;
    if (image->texture && ratio > 0) {
        const cui__cr_state *state = cui_state(ui, key, sizeof *state);
        fpas_constrain_crop(current, state, ratio, minimum, image->width, image->height);
        cui__cr_paint *paint = ui->nodes[first].paint_data;
        paint->crop = *current;
    }
    fpas_creative_node(ui, first, key);
    return enabled && memcmp(&previous, current, sizeof previous) != 0;
}
