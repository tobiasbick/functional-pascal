/* Checks native segment pixels, keyboard focus and preservation of the model palette. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int frame(cui_ctx *ui, int selected, int key, int enabled, int focus, bool with_system, int *nodes) {
    cui_begin_frame(ui, 0.04f);
    if (key) {
        cui_input_key(ui, (cui_key)key, true, 0);
        cui_input_key(ui, (cui_key)key, false, 0);
    }
    const char *labels[] = { "", "", "" };
    const char *icons[] = { CUI_ICON_SUN, CUI_ICON_MOON, CUI_ICON_MONITOR };
    const int proposal = fpas_cui_segmented(ui, "fixture", labels, icons, with_system ? 3 : 2,
        selected, CUI_SIZE_SM, 0, enabled, focus, nodes);
    cui_end_frame(ui);
    return proposal;
}

static bool verify(cui_ctx *ui) {
    uint32_t before[CUI_TOKEN_COUNT - 1], after[CUI_TOKEN_COUNT - 1];
    fpas_cui_theme_read(ui, before);
    before[0] = 0xff352719u;
    before[4] = 0xffc7d9ebu;
    fpas_cui_theme(ui, 1, 2, before);
    int nodes[4];
    if (frame(ui, 1, 0, 1, 1, true, nodes) != -1) return false;
    if (fpas_cui_last_node(ui) != nodes[0]) return false;
    if (fpas_cui_node_rect(ui, nodes[0]).h != 30) return false;
    const unsigned int light = fpas_cui_node_key(ui, nodes[1]);
    if (!fpas_cui_focus_key(ui, light)) return false;
    if (frame(ui, 1, CUI_KEY_ENTER, 1, 0, true, nodes) != 0) return false;
    if (frame(ui, 1, 0, 1, 0, true, nodes) != -1) return false;
    if (frame(ui, 1, CUI_KEY_RIGHT, 1, 0, true, nodes) != -1) return false;
    if (!fpas_cui_focus_key(ui, fpas_cui_node_key(ui, nodes[2]))) return false;
    if (frame(ui, 1, CUI_KEY_RIGHT, 1, 0, true, nodes) != 2) return false;
    if (!fpas_cui_focus_key(ui, fpas_cui_node_key(ui, nodes[3]))) return false;
    if (frame(ui, 2, CUI_KEY_RIGHT, 1, 0, true, nodes) != -1) return false;
    if (frame(ui, 2, CUI_KEY_SPACE, 1, 0, true, nodes) != -1) return false;
    if (frame(ui, 2, CUI_KEY_LEFT, 1, 0, true, nodes) != 1) return false;
    if (frame(ui, 1, CUI_KEY_LEFT, 0, 0, true, nodes) != -1) return false;
    if (fpas_cui_focus_key(ui, fpas_cui_node_key(ui, nodes[2]))) return false;
    if (frame(ui, 1, 0, 1, 3, true, nodes) != -1) return false;
    if (frame(ui, 1, 0, 1, 0, false, nodes) != -1) return false;
    if (fpas_cui_focus_key(ui, fpas_cui_node_key(ui, nodes[1]))) return false;
    fpas_cui_theme_read(ui, after);
    return memcmp(before, after, sizeof before) == 0 && cui_get_theme_mode(ui) == CUI_THEME_DARK;
}

static bool pixel_parity(int size, bool full_width, bool with_icons) {
    const int width = 640, height = 120;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *expected = calloc(1, bytes), *actual = calloc(1, bytes);
    if (!expected || !actual) { free(expected); free(actual); return false; }
    bool valid = true;
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        if (!renderer) { valid = false; break; }
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); valid = false; break; }
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        const char *labels[] = { "List", "Board", "Timeline" };
        const char *icons[] = { CUI_ICON_LIST, CUI_ICON_LAYOUT_DASHBOARD, CUI_ICON_HISTORY };
        for (int frame_number = 0; frame_number < 3; ++frame_number) {
            cui_begin_frame(ui, 0.04f);
            cui_column(ui, .width = CUI_FILL, .pad = 24) {
                int selected = 1;
                if (pass) {
                    int nodes[4];
                    fpas_cui_segmented(ui, "parity", labels, with_icons ? icons : NULL,
                        3, selected, size, full_width, 1, 0, nodes);
                } else {
                    const cui_segmented_opts options = { .size = (cui_control_size)size,
                        .full_width = full_width, .icons = with_icons ? icons : NULL };
                    cui_segmented_ex(ui, "parity", labels, 3, &selected, &options);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pass ? actual : expected,
                width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(expected, actual, bytes) == 0;
    free(expected); free(actual);
    return valid;
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return 1;
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    if (!ui) { cui_sw_destroy(renderer); return 1; }
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    bool passed = verify(ui);
    for (int size = CUI_SIZE_MD; size <= CUI_SIZE_LG; ++size)
        for (int fill = 0; fill < 2; ++fill)
            for (int icons = 0; icons < 2; ++icons)
                passed = pixel_parity(size, fill != 0, icons != 0) && passed;
    cui_destroy(ui); cui_sw_destroy(renderer);
    if (!passed) fprintf(stderr, "Segments changed native pixels or palette, lost focus or ignored controlled input.\n");
    return passed ? 0 : 1;
}
