/* Capacity, threshold, zero-capacity and animation output must remain native. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int sample, bool decorated, bool dark, int width) {
    const float values[] = { 0, 7.5f, 7.5001f, 9, 9.0001f, 12, 0, 4 };
    const float total = sample < 6 ? 10 : 0;
    const char *label = decorated ? "Storage" : "";
    const char *unit = decorated ? "GB ## literal" : "";
    const int height = 160;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&options);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 6; ++frame) {
            const float used = frame < 2 ? 0 : values[sample];
            cui_begin_frame(ui, .04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16) {
                if (pass) fpas_cui_meter(ui, "###meter", label, used, total, unit);
                else cui_meter(ui, label, used, total, *unit ? unit : NULL);
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Meter pixel mismatch: sample=%d decorated=%d dark=%d width=%d\n",
        sample, decorated, dark, width);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int sample = 0; sample < 8; ++sample)
        for (int decorated = 0; decorated < 2; ++decorated)
            for (int dark = 0; dark < 2; ++dark)
                for (int narrow = 0; narrow < 2; ++narrow)
                    valid = pixels_match(sample, decorated != 0, dark != 0, narrow ? 280 : 640) && valid;
    return valid ? 0 : 1;
}
