/* Native widgets and their ABI adapters must produce identical pixels. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool pixels_match(int kind, int tone, bool decorated, bool dark, int width) {
    const int height = 440;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16, .gap = 8) {
                if (kind == 0) {
                    const cui_alert_opts options = { .tone = (cui_tone)tone,
                        .description = decorated ? "Supporting ## literal description wraps across narrow space." : NULL,
                        .icon = decorated ? CUI_ICON_STAR : NULL, .dismissible = decorated };
                    if (pass) {
                        int node, dismiss; unsigned int key;
                        valid = fpas_cui_alert(ui, "###alert", "Native heading", options.description ? options.description : "",
                            options.icon ? options.icon : "", tone, decorated, 1, 0, &node, &dismiss, &key) == 1 && valid;
                    } else valid = cui_alert_ex(ui, "Native heading", &options) && valid;
                } else if (kind == 1) {
                    const cui_progress_opts options = { .tone = (cui_tone)tone,
                        .label = decorated ? "Uploading assets" : NULL, .show_value = decorated,
                        .indeterminate = !decorated, .height = decorated ? 12 : 0 };
                    if (pass) fpas_cui_progress(ui, "###progress", .68f, tone, !decorated,
                        options.height, options.label ? options.label : "", decorated);
                    else cui_progress_ex(ui, .68f, &options);
                } else {
                    if (pass) fpas_cui_spinner(ui, "###spinner", decorated ? 28 : 0);
                    else cui_spinner(ui, decorated ? 28 : 0);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Feedback pixel mismatch: kind=%d tone=%d decorated=%d dark=%d width=%d\n",
        kind, tone, decorated, dark, width);
    return valid;
}

/* Native painting, not a managed timer, must request activity frames. */
static bool activity_requests_frames(int kind) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    if (!ui) { cui_sw_destroy(renderer); return false; }
    cui_set_viewport(ui, 320, 160, 1);
    bool active = false;
    for (int frame = 0; frame < 32; ++frame) {
        cui_begin_frame(ui, .04f);
        cui_column(ui, .width = CUI_FILL, .pad = 16) {
            if (frame < 24) {
                if (kind) fpas_cui_spinner(ui, "###spin", 22);
                else fpas_cui_progress(ui, "###progress", 0, 0, 1, 0, "Indexing", 0);
            } else cui_text(ui, "Finished");
        }
        cui_end_frame(ui);
        if (frame == 23) active = cui_needs_redraw(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    if (!active || !stopped) fprintf(stderr, "Native activity wakeup mismatch: kind=%d\n", kind);
    return active && stopped;
}

int main(void) {
    bool valid = true;
    for (int kind = 0; kind < 3; ++kind)
        for (int tone = 0; tone < (kind == 2 ? 1 : 9); ++tone)
            for (int decorated = 0; decorated < 2; ++decorated)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = pixels_match(kind, tone, decorated != 0, dark != 0,
                            narrow ? 280 : 640) && valid;
    return activity_requests_frames(0) && activity_requests_frames(1) && valid ? 0 : 1;
}
