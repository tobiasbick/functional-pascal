/* Activity layout, literal text, native colors and the last-item connector stay in CUI. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int tone, int fields, bool last, bool dark, int width) {
    const int height = 240;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    const char *time = fields & 1 ? "2m ## ago" : "";
    const char *description = fields & 2
        ? "Release ## notes: native text wraps at the available width without a second layout engine." : "";
    const char *icon = fields & 4 ? CUI_ICON_ROCKET : "";
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&options);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 14) {
            if (pass) {
                fpas_cui_timeline_item(ui, "###entry", "Deployed ## literally", time, description, icon, tone, last);
                valid = valid && cui_last_id(ui) == cui_make_id(ui, "###entry");
            } else {
                const cui_timeline_opts settings = {
                    .time = *time ? time : NULL, .description = *description ? description : NULL,
                    .icon = *icon ? icon : NULL, .tone = (cui_tone)tone, .last = last
                };
                cui_timeline_item_ex(ui, "Deployed ## literally", &settings);
            }
        }
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Timeline mismatch: tone=%d fields=%d last=%d dark=%d width=%d\n",
        tone, fields, last, dark, width);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int tone = 0; tone < 9; ++tone)
        for (int fields = 0; fields < 8; ++fields)
            for (int last = 0; last < 2; ++last)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = pixels_match(tone, fields, last != 0, dark != 0,
                            narrow ? 280 : 640) && valid;
    return valid ? 0 : 1;
}
