/* The binding must preserve native tour painting and leave its base view's last node intact. */
#include "../fpas_cui.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

static bool matches(int selected, bool target, bool dark, bool block, int width) {
    const int height = 420;
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
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 3; ++frame) {
            cui_begin_frame(ui, .04f);
            cui_column(ui, .width = CUI_FILL, .pad = 24) { cui_skeleton(ui, CUI_PX(80), 80); }
            const int saved = fpas_cui_last_node(ui);
            const unsigned int key = target ? cui_last_id(ui) : 0;
            const unsigned int targets[3] = { key, key, key };
            const char *titles[3] = { "Turn ## literally", "Watch the levels", "Ride the faders" };
            const char *bodies[3] = { "Drag up and down. Shift gives fine steps.", "Native peak hold.", "Use the wheel or arrow keys." };
            const int positions[3] = { CUI_PLACE_BOTTOM_START, CUI_PLACE_TOP_START, CUI_PLACE_LEFT_START };
            if (frame > 0) {
                int step = selected;
                if (pass) {
                    int node, parts[3]; unsigned int owner;
                    fpas_cui_tour(ui, "###guide", targets, titles, bodies, positions, 3, &step,
                        block ? 260 : 0, block ? 9 : 0, block, 0, &node, &owner, parts);
                    valid = valid && node >= 0 && parts[0] >= 0 && parts[2] >= 0 &&
                        (parts[1] >= 0) == (selected > 0) && owner == cui_make_id(ui, "###guide") &&
                        fpas_cui_last_node(ui) == saved;
                } else {
                    const cui_tour_step steps[3] = {
                        { key, titles[0], bodies[0], (cui_placement)positions[0] },
                        { key, titles[1], bodies[1], (cui_placement)positions[1] },
                        { key, titles[2], bodies[2], (cui_placement)positions[2] }
                    };
                    const cui_tour_opts settings = { .width = block ? 260 : 0, .padding = block ? 9 : 0, .block_target = block };
                    cui_tour_ex(ui, "###guide", steps, 3, &step, &settings);
                }
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], width, height, width * 4, CUI_SW_RGBA);
        }
        cui_destroy(ui); cui_sw_destroy(renderer);
    }
    valid = valid && memcmp(pixels[0], pixels[1], bytes) == 0;
    free(pixels[0]); free(pixels[1]);
    if (!valid) fprintf(stderr, "Tour mismatch: step=%d target=%d dark=%d block=%d width=%d\n", selected, target, dark, block, width);
    return valid;
}

int main(void) {
    bool valid = true;
    for (int step = 0; step < 3; ++step)
        for (int target = 0; target < 2; ++target)
            for (int dark = 0; dark < 2; ++dark)
                for (int block = 0; block < 2; ++block)
                    for (int narrow = 0; narrow < 2; ++narrow)
                        valid = matches(step, target != 0, dark != 0, block != 0, narrow ? 280 : 640) && valid;
    return valid ? 0 : 1;
}
