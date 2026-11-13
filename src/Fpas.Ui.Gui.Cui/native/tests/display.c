#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

static bool close_to(float actual, float expected) {
    return fabsf(actual - expected) < 0.01f;
}

static bool check_canvas(bool covered, bool bridge) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 180, 1);
    cui_set_reduced_motion(ui, true);
    const fpas_canvas_shape shape = {1, 15, 25, 0, 0, 6, 0xff112233u};
    int activations = 0;
    bool coordinates = false;
    for (int frame = 0; frame < 6; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 31, 41);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        cui_box(ui, .dir = CUI_STACK, .pad = 16) {
            float x, y, w, h;
            if (fpas_cui_drawing_canvas(ui, "canvas", 2, 80, 2, 60, &shape, 1, 1, &x, &y, &w, &h)) {
                ++activations;
                coordinates = close_to(x, 15) && close_to(y, 25) && close_to(w, 80) && close_to(h, 60);
            }
            if (covered) {
                if (bridge) {
                    fpas_cui_veil_begin(ui, "disabled-cover", 0, NULL, NULL, 0, 0, 2, 80, 0);
                    cui_box(ui, .width = CUI_PX(80), .height = CUI_PX(60)) {}
                    fpas_cui_veil_end(ui);
                } else {
                    bool shown = false;
                    cui_push_disabled(ui, true);
                    cui_veil(ui, "disabled-cover", &shown, .width = CUI_PX(80)) {
                        cui_box(ui, .width = CUI_PX(80), .height = CUI_PX(60)) {}
                    }
                    cui_pop_disabled(ui);
                }
            }
        }
        cui_end_frame(ui);
    }
    cui_destroy(ui);
    if (activations == 1 && coordinates) return true;
    fprintf(stderr, "Expected one canvas activation at (15,25) in an 80x60 area; got %d.\n", activations);
    return false;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return check_canvas(true, false) ? 0 : 1;
    return check_canvas(false, true) && check_canvas(true, true) ? 0 : 1;
}
