#include "../fpas_cui.h"
#include <stdio.h>
#include <math.h>

static int failures;

static void check(bool value, const char *message) {
    if (value) return;
    fprintf(stderr, "%s\n", message);
    failures++;
}

static cui_ctx *canvas(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return NULL;
    cui_set_viewport(ui, 650, 440, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void click(cui_ctx *ui, float x, float y, bool down) {
    cui_input_mouse_move(ui, x, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, down);
}

static void graph_pan_and_move(bool bridge, bool enabled) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create graph context."); return; }
    cui_node_view view = { 10, 6, 1 };
    float x = 36, y = 64;
    for (int k = 0; k < 8; ++k) {
        if (k == 2) click(ui, 70, 83, true);
        if (k == 3) cui_input_mouse_move(ui, 115, 110);
        if (k == 4) click(ui, 115, 110, false);
        if (k == 6) {
            cui_input_mouse_move(ui, 350, 140);
            cui_input_scroll(ui, 0, 1);
        }
        cui_begin_frame(ui, .016f);
        if (!bridge) cui_push_disabled(ui, !enabled);
        const bool opened = bridge
            ? fpas_cui_node_editor_begin(ui, "graph", view, 1, 0, 2, 320, 0, 0, enabled) != 0
            : cui_node_editor_begin(ui, "graph", &view, .height = CUI_PX(320));
        check(opened,
            "Could not build the node editor.");
        fpas_cui_node_begin(ui, 12, &x, &y, "Source", 184, NULL);
        fpas_cui_node_pin(ui, 8, "Output", 1, 1);
        cui_node_end(ui);
        if (bridge) fpas_cui_node_editor_end(ui, &view);
        else { cui_node_editor_end(ui); cui_pop_disabled(ui); }
        cui_end_frame(ui);
    }
    if (enabled) {
        check(x == 81 && y == 91, "Dragging the node must propose its exact canvas position.");
        check(view.zoom > 1 && view.x < 10, "Wheel zoom must keep the point under the pointer fixed.");
    } else {
        check(x == 36 && y == 64 && view.zoom == 1 && view.x == 10 && view.y == 6,
            "Disabled node editor must ignore dragging and wheel zoom.");
    }
    cui_destroy(ui);
}

static void canvas_rail(void) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create canvas rail context."); return; }
    cui_node_view view = { 10, 6, 1 };
    const fpas_canvas_toggle toggle = { CUI_ICON_EYE, "Show grid" };
    bool grid = true;
    int events = 0;
    for (int k = 0; k < 12; ++k) {
        if (k == 3 || k == 4) click(ui, 615, 83, k == 3);
        if (k == 7 || k == 8) click(ui, 615, 115, k == 7);
        cui_begin_frame(ui, .016f);
        cui_box(ui, .width = CUI_FILL, .height = CUI_PX(320)) {}
        const int event = fpas_cui_canvas_controls(ui, "rail", &view, (cui_rect){0},
            NULL, 0, &toggle, &grid, 1, 1);
        events += (event & CUI_CANVAS_ZOOMED) != 0;
        if (k == 5) check(view.zoom == 1.25f, "Zoom in must select the next preset.");
        cui_end_frame(ui);
    }
    check(events == 2 && view.zoom == 1, "The zoom readout must restore 100 percent.");
    cui_destroy(ui);
}

static void format_and_cycle(void) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create toolbar context."); return; }
    cui_text_format format = { CUI_TONE_ROSE, true, false, false, 0, 44, CUI_FORMAT_ALIGN_LEFT, CUI_FORMAT_LIST_NONE };
    const char *types[] = { "PNG", "JPG", "WEBP" };
    int current = 0;
    int activated = 0;
    int formats = 0;
    for (int k = 0; k < 8; ++k) {
        if (k == 2) click(ui, 20, 77, true);
        if (k == 3) click(ui, 20, 77, false);
        if (k == 5 || k == 6) click(ui, 88, 23, k == 5);
        cui_begin_frame(ui, .016f);
        cui_column(ui, .gap = 12, .width = CUI_FILL) {
            const int result = fpas_cui_format_bar(ui, "format", &format, NULL, 0, NULL, 0, NULL, NULL, NULL, 1);
            if (result & CUI_FORMAT_CHANGED) formats++;
            fpas_cui_toolbar_begin(ui, "tools", CUI_TOOLBAR_FLOATING, 0, 1);
            int next = fpas_cui_toolbar_cycle(ui, "type", types, 3, current, "Format");
            if (next >= 0) { current = next; activated++; }
            fpas_cui_toolbar_end(ui);
        }
        cui_end_frame(ui);
    }
    check(current == 1 && activated == 1, "Activating a cycle tool must advance exactly one native option.");
    check(formats == 1 && format.color == CUI_TONE_ROSE && format.size == 44 && !format.bold,
        "The Bold button must toggle only the selected formatting field.");
    cui_destroy(ui);
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) {
        graph_pan_and_move(false, false);
        return failures != 0;
    }
    graph_pan_and_move(true, true);
    graph_pan_and_move(true, false);
    format_and_cycle();
    canvas_rail();
    return failures != 0;
}
