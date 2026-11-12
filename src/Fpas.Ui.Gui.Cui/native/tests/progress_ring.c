/* Ring painting, independent identities, changing captions and state retirement stay native. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static bool reset(cui_ctx **ui, cui_sw_renderer **renderer, int width, bool dark) {
    if (*ui) cui_destroy(*ui);
    if (*renderer) cui_sw_destroy(*renderer);
    *renderer = cui_sw_create();
    const cui_ctx_desc options = { .renderer = cui_sw_iface(*renderer) };
    *ui = cui_create(&options);
    if (!*ui) return false;
    cui_set_viewport(*ui, width, 200, 1);
    /* Settle only the palette, so a fresh reference and retained context use
       identical colors while ring interpolation remains enabled below. */
    cui_set_reduced_motion(*ui, true);
    cui_set_theme(*ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    cui_begin_frame(*ui, 0);
    cui_end_frame(*ui);
    cui_set_reduced_motion(*ui, false);
    return true;
}

static bool pixels_match(float value, float size, bool labeled, bool dark, int width, bool retire) {
    const size_t bytes = (size_t)width * 200 * 4;
    unsigned char *pixels[2] = { calloc(1, bytes), calloc(1, bytes) };
    if (!pixels[0] || !pixels[1]) { free(pixels[0]); free(pixels[1]); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = NULL;
        cui_ctx *ui = NULL;
        if (!reset(&ui, &renderer, width, dark)) { cui_sw_destroy(renderer); valid = false; break; }
        cui_id first = 0;
        for (int frame = 0; frame < 6; ++frame) {
            if (retire && frame == 4) {
                if (pass) fpas_cui_forget(ui, first);
                else if (!reset(&ui, &renderer, width, dark)) { valid = false; break; }
            }
            const char *label = !labeled ? "" : frame < 2 ? "Before" : "Onboarding";
            char caption[64];
            const float fraction = frame < 2 ? 0 : value;
            cui_begin_frame(ui, .04f);
            cui_column(ui, .width = CUI_FILL, .pad = 16) {
                if (frame >= 2) cui_text(ui, "Goals");
                cui_row(ui, .gap = 8) {
                    if (pass) {
                        fpas_cui_progress_ring(ui, "###first", label, fraction, size);
                        first = cui_last_id(ui);
                        fpas_cui_progress_ring(ui, "###second", label, 1 - value, size);
                    } else {
                        snprintf(caption, sizeof caption, "%s###first", label);
                        cui_progress_ring(ui, caption, fraction, size);
                        snprintf(caption, sizeof caption, "%s###second", label);
                        cui_progress_ring(ui, caption, 1 - value, size);
                    }
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, 200, width * 4, CUI_SW_RGBA);
        }
        if (ui) cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Ring mismatch: value=%g size=%g label=%d dark=%d width=%d retire=%d\n",
        value, size, labeled, dark, width, retire);
    return valid;
}

int main(void) {
    const float values[] = { 0, .35f, .72f, 1 };
    const float sizes[] = { 0, 64, 104 };
    bool valid = true;
    for (int value = 0; value < 4; ++value)
        for (int size = 0; size < 3; ++size)
            for (int label = 0; label < 2; ++label)
                for (int dark = 0; dark < 2; ++dark)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        for (int retire = 0; retire < 2; ++retire)
                            valid = pixels_match(values[value], sizes[size], label != 0, dark != 0,
                                narrow ? 280 : 640, retire != 0) && valid;
    return valid ? 0 : 1;
}
