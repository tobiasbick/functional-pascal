/* Drives CUI's status button with real input and deterministic frame time. */
#include "../fpas_cui.h"
#include <stdio.h>

static int frame(cui_ctx *ui, int status, int key, int enabled, int focus, const char *label) {
    cui_begin_frame(ui, 0.04f);
    if (key) {
        cui_input_key(ui, (cui_key)key, true, 0);
        cui_input_key(ui, (cui_key)key, false, 0);
    }
    const int activated = fpas_cui_status_button(ui, "###status-fixture", label, status,
        "Saving ## literally", "Saved ### literally", "Failed ## literally", NULL, 1, 0, enabled, focus);
    cui_end_frame(ui);
    return activated;
}

static bool verify(cui_ctx *ui) {
    if (frame(ui, 0, 0, 1, 1, "Save ## literally")) return false;
    const unsigned int key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    if (!key || !fpas_cui_focus_key(ui, key) || cui_last_rect(ui).w < 100) return false;
    if (!frame(ui, 0, CUI_KEY_ENTER, 1, 0, "Save")) return false;
    for (int status = 1; status <= 3; ++status) {
        if (frame(ui, status, CUI_KEY_SPACE, 1, 0, "Renamed ### literally")) return false;
        if (fpas_cui_node_key(ui, fpas_cui_last_node(ui)) != key || !fpas_cui_focus_key(ui, key)) return false;
        for (int i = 0; i < 100; ++i) if (frame(ui, status, 0, 1, 0, "Save")) return false;
        if (frame(ui, status, CUI_KEY_ENTER, 1, 0, "Save")) return false;
    }
    if (frame(ui, 0, CUI_KEY_ENTER, 0, 0, "Save")) return false;
    if (fpas_cui_focus_key(ui, key)) return false;
    if (!frame(ui, 0, CUI_KEY_SPACE, 1, 1, "Save")) return false;

    const cui_rect bounds = cui_last_rect(ui);
    cui_begin_frame(ui, 0.04f);
    fpas_cui_move_pointer(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    const int pressed = fpas_cui_status_button(ui, "###status-fixture", "Save", 0, "Busy", "Done", "Failed", NULL, 1, 0, 1, 0);
    cui_end_frame(ui);
    if (pressed) return false;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    if (!frame(ui, 0, 0, 1, 0, "Save")) return false;
    if (frame(ui, 0, 0, 1, 0, "Save")) return false;
    cui_set_reduced_motion(ui, false);
    for (int i = 0; i < 100; ++i) frame(ui, 2, 0, 1, 0, "Save ## literally");
    if (cui_needs_redraw(ui)) return false;
    return frame(ui, 2, CUI_KEY_ENTER, 1, 0, "Save") == 0;
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
    if (!passed) fprintf(stderr, "Status did not preserve idle-only activation, model state, geometry or focus.\n");
    return passed ? 0 : 1;
}
