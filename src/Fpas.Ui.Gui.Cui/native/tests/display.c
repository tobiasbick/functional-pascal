// [Documentation](../../../../docs/ui/gui/text-and-display/README.md)
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
            float dx, dy;
            if (fpas_cui_drawing_canvas(ui, "canvas", 2, 80, 2, 60, &shape, 1, 1, &x, &y, &w, &h, 0, &dx, &dy, NULL, NULL, 0, 0, 0, 0, CUI_CURSOR_HAND) & 1) {
                ++activations;
                coordinates = close_to(x, 15) && close_to(y, 25) && close_to(w, 80) && close_to(h, 60) && cui_get_cursor(ui) == CUI_CURSOR_HAND;
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

static bool check_literal_link(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 280, 180, 1);
    const char *labels[] = {"Docs", "Docs#", "#"};
    cui_id identity = 0;
    float plain_width = 0;
    bool ok = true;
    for (int frame = 0; frame < 3; ++frame) {
        cui_begin_frame(ui, .016f);
        int node;
        fpas_cui_link(ui, "###docs", labels[frame], 1, frame == 0, &node);
        cui_end_frame(ui);
        const cui_id key = fpas_cui_node_key(ui, node);
        const float width = fpas_cui_node_rect(ui, node).w;
        if (frame == 0) {
            identity = key;
            plain_width = width;
        }
        if (!key || key != identity || !fpas_cui_focused(ui, node) || width <= 0 ||
            (frame == 1 && width <= plain_width)) ok = false;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "A trailing # changed the link's caption or focused identity.\n");
    return ok;
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return check_canvas(true, false) ? 0 : 1;
    return check_canvas(false, true) && check_canvas(true, true) && check_literal_link() ? 0 : 1;
}
