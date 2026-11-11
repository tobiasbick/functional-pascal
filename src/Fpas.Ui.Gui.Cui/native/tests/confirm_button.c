/* Exercises CUI confirmation with deterministic frame time and real input. */
#include "../fpas_cui.h"
#include <stdio.h>

static int frame(cui_ctx *ui, int key, int enabled, int focus, int timeout, const char *label) {
    cui_begin_frame(ui, 0.04f);
    if (key) {
        cui_input_key(ui, (cui_key)key, true, 0);
        cui_input_key(ui, (cui_key)key, false, 0);
    }
    const int events = fpas_cui_confirm_button(ui, "###confirm-fixture", label,
        "Confirm ## literally", NULL, timeout, 2, 0, enabled, focus);
    cui_end_frame(ui);
    return events;
}

static bool verify(cui_ctx *ui) {
    if (frame(ui, 0, 1, 1, 3000, "Delete ## literally") != 0) return false;
    const unsigned int key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    if (!key || !fpas_cui_focus_key(ui, key)) return false;
    const float width = cui_last_rect(ui).w;
    if (frame(ui, CUI_KEY_ENTER, 1, 0, 3000, "Renamed ### literally") != 2) return false;
    if (key != fpas_cui_node_key(ui, fpas_cui_last_node(ui)) || !fpas_cui_focus_key(ui, key)) return false;
    if (frame(ui, CUI_KEY_SPACE, 1, 0, 3000, "Delete") != 1) return false;
    if (frame(ui, 0, 1, 0, 3000, "Delete ## literally") != 0) return false;
    if (cui_last_rect(ui).w != width) return false;

    if (frame(ui, CUI_KEY_ENTER, 1, 0, 3000, "Delete") != 2) return false;
    for (int i = 0; i < 73; ++i) if (frame(ui, 0, 1, 0, 3000, "Delete") != 2) return false;
    for (int i = 0; i < 4; ++i) frame(ui, 0, 1, 0, 3000, "Delete");
    if (frame(ui, CUI_KEY_ENTER, 1, 0, 3000, "Delete") != 2) return false;
    if (frame(ui, CUI_KEY_ESCAPE, 1, 0, 3000, "Delete") != 0) return false;

    if (frame(ui, CUI_KEY_ENTER, 1, 0, 120, "Delete") != 2) return false;
    for (int i = 0; i < 5; ++i) frame(ui, 0, 1, 0, 120, "Delete");
    if (frame(ui, CUI_KEY_ENTER, 1, 0, 120, "Delete") != 2) return false;
    if (frame(ui, CUI_KEY_SPACE, 0, 0, 120, "Delete") != 0) return false;
    if (fpas_cui_focus_key(ui, key)) return false;
    if (frame(ui, CUI_KEY_ENTER, 1, 1, 3000, "Delete") != 2) return false;

    cui_begin_frame(ui, 0.04f);
    fpas_cui_move_pointer(ui, 600, 400);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    const int outside = fpas_cui_confirm_button(ui, "###confirm-fixture", "Delete", "Confirm", NULL, 3000, 2, 0, 1, 0);
    cui_end_frame(ui);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    if (outside != 0) return false;
    if (frame(ui, CUI_KEY_ENTER, 1, 1, 3000, "Delete") != 2) return false;
    frame(ui, CUI_KEY_ESCAPE, 1, 0, 3000, "Delete");
    cui_set_reduced_motion(ui, false);
    for (int i = 0; i < 100; ++i) frame(ui, 0, 1, 0, 3000, "Delete ## literally");
    return !cui_needs_redraw(ui);
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    if (!renderer) return 1;
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    if (!ui) { cui_sw_destroy(renderer); return 1; }
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool passed = verify(ui);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    if (!passed) fprintf(stderr, "Confirmation did not preserve two-step activation, timing, cancellation or focus.\n");
    return passed ? 0 : 1;
}
