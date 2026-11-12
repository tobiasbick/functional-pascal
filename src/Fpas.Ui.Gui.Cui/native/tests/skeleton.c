/* Native shimmer, sizes, reduced motion and percentage button widths must match CUI. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int sample, float height, bool reduced, bool dark, int width) {
    const int kinds[] = { 1, 2, 3, 3, 3, 3 };
    const float values[] = { 0, 40, 0, 45, 70, 100 };
    const int kind = kinds[sample];
    const float value = values[sample];
    const size_t bytes = (size_t)width * 180 * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&options);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, 180, 1);
        cui_set_reduced_motion(ui, reduced);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 12; ++frame) {
            cui_begin_frame(ui, .1f);
            cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
                if (pass) fpas_cui_skeleton(ui, "###skeleton", kind, value, height);
                else cui_skeleton(ui, kind == 1 ? CUI_FILL : kind == 2 ? CUI_PX(value) : CUI_PCT(value), height);
                if (kind == 3) {
                    if (pass) {
                        fpas_cui_button(ui, "###button", "Example", 0, 0, 1, 0, 0, 0,
                            NULL, NULL, NULL, NULL, 0, -1, 0);
                        fpas_cui_node_size(ui, fpas_cui_last_node(ui), 1, 3, value, 0, -1, 0, 0);
                    } else {
                        const cui_button_opts button = { .width = CUI_PCT(value) };
                        cui_button_ex(ui, "Example", &button);
                    }
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, 180, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Skeleton mismatch: sample=%d height=%g reduced=%d dark=%d width=%d\n",
        sample, height, reduced, dark, width);
    return valid;
}

static bool activity(bool reduced) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, 300, 160, 1);
    cui_set_reduced_motion(ui, reduced);
    bool moving = false;
    for (int frame = 0; frame < 32; ++frame) {
        cui_begin_frame(ui, .04f);
        if (frame < 24) fpas_cui_skeleton(ui, "###loading", 1, 0, 40);
        cui_end_frame(ui);
        if (frame == 23) moving = cui_needs_redraw(ui);
    }
    const bool valid = moving == !reduced && !cui_needs_redraw(ui);
    cui_destroy(ui); cui_sw_destroy(renderer);
    return valid;
}

int main(void) {
    const float heights[] = { 0, 12, 88 };
    bool valid = true;
    for (int sample = 0; sample < 6; ++sample)
        for (int height = 0; height < 3; ++height)
            for (int reduced = 0; reduced < 2; ++reduced)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = pixels_match(sample, heights[height], reduced != 0, dark != 0,
                            narrow ? 280 : 640) && valid;
    return activity(false) && activity(true) && valid ? 0 : 1;
}
