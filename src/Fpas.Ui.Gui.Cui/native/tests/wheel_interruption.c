// [Documentation](../../../../docs/ui/gui/pickers/wheel-picker.md)
/* Adapter-level black-box drag interruption probe; no private native state inspection. */
#include "../fpas_cui.h"
#include <stdio.h>

/** Proves a wheel drag works, then checks that disabling cancels the held gesture. */
int main(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 400, 400, 1);
    cui_set_reduced_motion(ui, true);
    const char *items[] = {
        "00", "01", "02", "03", "04", "05", "06", "07", "08", "09",
        "10", "11", "12", "13", "14", "15", "16", "17", "18", "19"
    };
    int selected = 10, node = -1, interrupted = -1;
    float x = 0, y = 0, row = 0;
    bool ok = true;
    for (int frame = 0; frame < 15; ++frame) {
        if (frame == 2 || frame == 10) {
            if (node < 0) { ok = false; break; }
            const cui_rect rect = cui_get_rect(ui, fpas_cui_node_key(ui, node));
            if (rect.w <= 0 || rect.h <= 0) { ok = false; break; }
            x = rect.x + rect.w / 2;
            y = rect.y + rect.h / 2;
            row = rect.h / 5;
            cui_input_mouse_move(ui, x, y);
        }
        if (frame == 3 || frame == 11) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 4) cui_input_mouse_move(ui, x, y - row);
        if (frame == 5) cui_input_mouse_move(ui, x, y - 2 * row);
        if (frame == 6 || frame == 7) cui_input_mouse_move(ui, x, y - (frame - 3) * row);
        if (frame == 8 || frame == 13) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 12) cui_input_mouse_move(ui, x, y - row);
        cui_begin_frame(ui, .016f);
        selected = fpas_cui_wheel_picker(ui, "wheel", items, 20, selected, 160, 5,
            0, 0, 0, frame != 5, 0, &node);
        cui_end_frame(ui);
        if (frame == 4) {
            if (selected == 10) {
                fprintf(stderr, "Wheel probe did not establish an actual drag.\n");
                ok = false; break;
            }
            interrupted = selected;
        }
        if (frame >= 5 && frame <= 9 && selected != interrupted) {
            fprintf(stderr, "Wheel resumed an interrupted drag at frame %d: %d -> %d.\n",
                frame, interrupted, selected);
            ok = false; break;
        }
        if (frame == 12 && selected == interrupted) {
            fprintf(stderr, "Wheel did not accept a fresh press after interruption.\n");
            ok = false; break;
        }
    }
    cui_destroy(ui);
    return ok ? 0 : 1;
}
