// [Documentation](../../../../../docs/ui/gui/canvas-and-bars/result-card.md)
#include "../../fpas_cui.h"

typedef struct {
    cui__result_paint *picture;
    float aspect;
    float layout_height;
} fpas_result_card_paint;

/** CUI-133 (docs/future/cui-upstream-bugs.md#cui-133): reconcile the height after the available width changes. */
static void fpas_paint_result_card(cui_ctx *ui, cui_node *node, void *data) {
    const fpas_result_card_paint *paint = data;
    cui__paint_result(ui, node, paint->picture);
    if (node->rect.w > 0 && roundf(node->rect.w * paint->aspect) != paint->layout_height)
        cui_request_redraw(ui);
}

/** Draws a result card and reports activation while it owns input. */
int fpas_cui_result_card(cui_ctx *ui, const char *id, const fpas_media_source *source, const char *title,
    const char *caption, const char *badge, int tone, int width_kind, float width, float aspect, int enabled,
    int focus, int stretch_image) {
    const cui_id key = cui_make_id(ui, id);
    const cui_id texture = source ? key ^ 0x52455355u : 0;
    fpas_own(ui, key, texture, 0, 0);
    const fpas_media_image *image = source ? fpas_media_resolve(ui, texture, source) : NULL;
    const cui_result_card_opts options = { .caption = caption, .badge = badge, .badge_tone = (cui_tone)tone,
        .image_w = image && !stretch_image ? image->width : 0,
        .image_h = image && !stretch_image ? image->height : 0,
        .width = fpas_extent(width_kind, width), .aspect = aspect };
    const bool accepts_input = enabled && !cui_is_disabled(ui);
    /* CUI-107 (docs/future/cui-upstream-bugs.md#cui-107): leave activation keys to the open popup. */
    const bool covered = accepts_input && cui__keys_covered(ui);
    const uint8_t enter = ui->kpressed[CUI_KEY_ENTER], space = ui->kpressed[CUI_KEY_SPACE];
    if (accepts_input && focus && !covered) ui->focus_next_request = key;
    if (covered) ui->kpressed[CUI_KEY_ENTER] = ui->kpressed[CUI_KEY_SPACE] = 0;
    cui_push_disabled(ui, !accepts_input);
    const bool activated = cui_result_card_ex(ui, id, image ? image->texture : 0, title, &options);
    cui_pop_disabled(ui);
    if (covered) {
        ui->kpressed[CUI_KEY_ENTER] = enter;
        ui->kpressed[CUI_KEY_SPACE] = space;
    }

    cui_node *node = &ui->nodes[ui->last_node];
    fpas_result_card_paint *paint = CUI_PAINT_DATA(ui, fpas_result_card_paint);
    *paint = (fpas_result_card_paint){ node->paint_data, aspect > 0 ? aspect : CUI__RESULT_ASPECT, node->h.value };
    node->paint = fpas_paint_result_card;
    node->paint_data = paint;
    return accepts_input && activated;
}
