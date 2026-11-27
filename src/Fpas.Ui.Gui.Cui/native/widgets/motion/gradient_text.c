// [Documentation](../../../../../docs/ui/gui/motion/gradient-text.md)
#include "../../fpas_cui.h"

static cui_color fpas_static_gradient(cui_ctx *ui, void *data, int byte,
    float x, float line_y, float *dy, cui_color color) {
    (void)byte; (void)line_y; (void)dy;
    const cui__grad_fx *gradient = data;
    const cui_rect bounds = ui->nodes[gradient->node].rect;
    const float position = bounds.w > 0 ? cui__clampf((x - bounds.x) / bounds.w, 0, 1) : 0;
    cui_color result = cui_mix(gradient->a, gradient->b, position);
    result.a = (uint8_t)(result.a * color.a / 255);
    return result;
}

void fpas_cui_gradient_text(cui_ctx *ui, const char *id, const char *text, const fpas_motion_text *o,
    int to_palette, uint32_t to_rgba, int animate) {
    const int node = ui->node_count;
    const cui_gradient_text_opts options = { .font = o->font, .size = o->size,
        .width = fpas_extent(o->width_kind, o->width), .from = fpas_chart_color(o->palette, o->rgba),
        .to = fpas_chart_color(to_palette, to_rgba), .animate = animate != 0 };
    cui_gradient_text_ex(ui, text, &options);
    /* [CUI-101](../../../../../docs/future/cui-upstream-bugs.md#cui-101): a static gradient goes from left to right. */
    if (!animate) ui->nodes[node].text_fx = fpas_static_gradient;
    fpas_motion_node(ui, node, cui_make_id(ui, id));
}
