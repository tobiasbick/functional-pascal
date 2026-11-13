#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static bool typing_stops_when_motion_is_reduced(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    for (int i = 0; i < 12; ++i) {
        cui_begin_frame(ui, 0.05f);
        if (adapter) fpas_cui_typing_indicator(ui, "typing", "Jonas");
        else cui_typing_indicator(ui, "Jonas");
        cui_end_frame(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    if (!stopped) fprintf(stderr, "Reduced-motion typing kept requesting frames.\n");
    return stopped;
}

static bool status_visibility(void) {
    const int states[] = { CUI_STATUS_IDLE, CUI_STATUS_BUSY, CUI_STATUS_SUCCESS, CUI_STATUS_SUCCESS,
        CUI_STATUS_ERROR, CUI_STATUS_ERROR };
    const float elapsed[] = { 0.01f, 0.05f, 0.05f, 4.0f, 0.05f, 4.0f };
    const int expected[] = { 0, 1, 1, 0, 1, 1 };
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        cui_begin_frame(ui, elapsed[i]);
        const int visible = fpas_cui_status_spinner(ui, "job", states[i], 18, 0, 0, i >= 4);
        cui_end_frame(ui);
        if (visible != expected[i]) ok = false;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Status spinner visibility did not follow idle, busy and result states.\n");
    return ok;
}

static bool native_pixels_match(void) {
    enum { WIDTH = 320, HEIGHT = 180 };
    static const float samples[] = { 3, 5, 4, 8, 6, 9, 8 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 4; ++frame) {
            cui_begin_frame(ui, 0.05f);
            cui_column(ui, .width = CUI_FILL, .gap = 8, .pad = 16) {
                if (pass) {
                    const int group = fpas_cui_skeleton_group_begin(ui, "group", 1, 0, 10, 1);
                    fpas_cui_skeleton(ui, "bar", 3, 65, 12);
                    fpas_cui_skeleton(ui, "line", 1, 0, 8);
                    fpas_cui_skeleton_group_end(ui, group);
                    fpas_cui_spinner_styled(ui, "spinner", 40, CUI_SPINNER_ORBIT, 0, 0, 0, 0);
                    fpas_cui_sparkline(ui, "trend", samples, 7, 0, 0, 0, 0, 0);
                } else {
                    cui_skeleton_group(ui, .dither = true, .gap = 10) {
                        cui_skeleton(ui, CUI_PCT(65), 12);
                        cui_skeleton(ui, CUI_FILL, 8);
                    }
                    cui_spinner(ui, 40, .style = CUI_SPINNER_ORBIT);
                    cui_sparkline(ui, samples, 7, CUI_FIT, 0, CUI_TRANSPARENT);
                }
            }
            cui_end_frame(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool equal = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!equal) fprintf(stderr, "Skeleton group, dither spinner or sparkline differs from direct CUI.\n");
    return equal;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return typing_stops_when_motion_is_reduced(false) ? 0 : 1;
    return typing_stops_when_motion_is_reduced(true) && status_visibility() && native_pixels_match() ? 0 : 1;
}
