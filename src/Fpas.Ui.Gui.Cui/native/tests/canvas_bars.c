// [Documentation](../../../../docs/ui/gui/canvas-and-bars/README.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <math.h>
#include <stdlib.h>
#include <string.h>

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

static void graph_reports_status(bool bridge) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create graph context."); return; }
    cui_node_view view = { 0, 0, 1 };
    float source_x = 35, source_y = 60, target_x = 350, target_y = 60;
    bool linked = true;
    for (int frame = 0; frame < 19; frame++) {
        if (frame == 2) cui_input_mouse_move(ui, 70, 73);
        if (frame == 3 || frame == 4) click(ui, 70, 73, frame == 3);
        if (frame == 5) cui_input_mouse_move(ui, 260, 113);
        if (frame == 6 || frame == 7) click(ui, 260, 113, frame == 6);
        if (frame == 8) cui_input_mouse_move(ui, 350, 113);
        if (frame == 9) click(ui, 350, 113, true);
        if (frame == 10) cui_input_mouse_move(ui, 300, 200);
        if (frame == 11) click(ui, 300, 200, false);
        if (frame == 14 || frame == 15) click(ui, 70, 73, frame == 14);
        if (frame == 16) cui_input_mouse_move(ui, 375, 73);
        if (frame == 17 || frame == 18) click(ui, 375, 73, frame == 17);
        cui_begin_frame(ui, .016f);
        const bool opened = bridge
            ? fpas_cui_node_editor_begin(ui, "graph", view, 1, 0, 2, 320, 0, 0, 1)
            : cui_node_editor_begin(ui, "graph", &view, .height = CUI_PX(320));
        check(opened, "Could not build the graph status editor.");
        const bool source_selected = bridge
            ? fpas_cui_node_begin(ui, 1, &source_x, &source_y, "Source", 140, CUI_ICON_IMAGE)
            : cui_node_begin(ui, 1, &source_x, &source_y, "Source", .width = 140, .icon = CUI_ICON_IMAGE);
        const bool output_connected = bridge
            ? fpas_cui_node_pin(ui, 10, "Output", 1, 1)
            : cui_node_output(ui, 10, "Output", .type = 1);
        cui_node_end(ui);
        if (bridge) fpas_cui_node_begin(ui, 2, &target_x, &target_y, "Target", 160, NULL);
        else cui_node_begin(ui, 2, &target_x, &target_y, "Target", .width = 160);
        const bool input_connected = bridge
            ? fpas_cui_node_pin(ui, 20, "Input", 0, 1)
            : cui_node_input(ui, 20, "Input", .type = 1);
        cui_node_content(ui) { cui_text(ui, "Node body"); }
        cui_node_end(ui);
        const bool link_selected = linked && cui_node_link(ui, 100, 10, 20);
        if (bridge) {
            if (frame == 17)
                check(source_selected && !fpas_cui_node_selected(ui, 1) && fpas_cui_node_selected(ui, 2),
                    "The final selection must reflect a later submitted node.");
            fpas_cui_node_editor_end(ui, &view);
        } else cui_node_editor_end(ui);
        int removed, unused;
        const bool unlinked = bridge
            ? fpas_cui_node_event(ui, 1, &removed, &unused)
            : cui_node_link_removed(ui, &removed);
        if (frame == 0 || frame == 10 || frame == 13)
            check(!input_connected && !output_connected, "Expected both native pin results to show no connection.");
        if (frame == 1)
            check(input_connected && output_connected, "Expected both pin directions to report the rendered link.");
        if (frame == 4)
            check(source_selected && !link_selected, "Clicking a node must return node selection.");
        if (frame == 7)
            check(link_selected && !source_selected, "Clicking a wire must return link selection.");
        check(unlinked == (frame == 11), "Detaching an input must report its removed link once.");
        if (unlinked) {
            check(removed == 100, "Removal must identify the detached link.");
            linked = false;
        }
        cui_end_frame(ui);
    }
    cui_destroy(ui);
}

/** The move and release can arrive together before the next rendered frame. */
static void graph_pan_release(void) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create a graph pan context."); return; }
    cui_node_view view = { 10, 6, 1 };
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1) cui_input_mouse_move(ui, 80, 100);
        if (frame == 2) cui_input_mouse_button(ui, CUI_MOUSE_MIDDLE, true);
        if (frame == 3) {
            cui_input_mouse_move(ui, 128, 132);
            cui_input_mouse_button(ui, CUI_MOUSE_MIDDLE, false);
        }
        cui_begin_frame(ui, .016f);
        check(fpas_cui_node_editor_begin(ui, "graph", view, 1, 0, 2, 320, 0, 0, 1),
            "Could not build the graph pan editor.");
        fpas_cui_node_editor_end(ui, &view);
        cui_end_frame(ui);
    }
    check(view.x == 58 && view.y == 38, "Middle-button pan must keep the movement queued with release.");
    cui_destroy(ui);
}

static void graph_selection_ring(void) {
    const int width = 650, height = 440;
    const size_t bytes = (size_t)width * height * 4;
    unsigned char *reference = malloc(bytes);
    unsigned char *actual = malloc(bytes);
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc desc = { .renderer = renderer ? cui_sw_iface(renderer) : (cui_renderer_iface){0} };
    cui_ctx *ui = renderer ? cui_create(&desc) : NULL;
    if (reference && actual && renderer && ui) {
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        for (int pass = 0; pass < 3; pass++) {
            cui_node_view view = { 0, 0, 1 };
            float x = 60, y = 70;
            cui_begin_frame(ui, .016f);
            if (cui_node_editor_begin(ui, "graph", &view, .height = CUI_PX(320), .no_grid = true)) {
                cui_node_begin(ui, 1, &x, &y, "Selected by a collaborator");
                cui_node_end(ui);
                if (pass == 0) cui_selection_ring(ui, ((cui_color){ 44, 90, 190, 255 }), .name = "Paul", .radius = 7);
                if (pass == 1) fpas_cui_selection_ring(ui, CUI_TONE_WARNING, 0xffbe5a2cu, "Paul", 7);
                cui_node_editor_end(ui);
            }
            cui_end_frame(ui);
            cui_sw_render(renderer, cui_get_draw_data(ui), pass == 0 ? reference : actual,
                width, height, width * 4, CUI_SW_RGBA);
            if (pass == 1)
                check(memcmp(reference, actual, bytes) == 0, "The bridge ring must match native color, name, and radius.");
            if (pass == 2)
                check(memcmp(reference, actual, bytes) != 0, "The native ring must draw on the completed card.");
        }
    } else {
        check(false, "Cannot create selection ring render context.");
    }
    if (ui) cui_destroy(ui);
    if (renderer) cui_sw_destroy(renderer);
    free(reference);
    free(actual);
}

typedef struct {
    cui_node_view view;
    cui_rect bounds;
    const float *steps;
    fpas_canvas_toggle toggle;
    int step_count, flags;
    bool pressed, rtl, popup;
} rail_case;

/** Builds an anchored rail and returns the current frame's action flags. */
static int rail_frame(cui_ctx *ui, rail_case *sample) {
    int event = 0;
    cui_begin_frame(ui, .016f);
    cui_box(ui, .direction = sample->rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR) {
        cui_button(ui, "Anchor");
        if (sample->popup) cui_popover_open(ui, "Options");
        cui_popover(ui, "Options", .width = 150) { cui_text(ui, "Overlay"); }
        cui_box(ui, .id = "rail-canvas", .width = CUI_PX(600), .height = CUI_PX(320)) {}
        const cui_id anchor = cui_last_id(ui);
        event = fpas_cui_canvas_controls(ui, "rail", &sample->view, sample->bounds,
            sample->steps, sample->step_count, &sample->toggle, &sample->pressed, 1, sample->flags);
        check(cui_last_id(ui) == anchor, "The rail must preserve the previous component as the overlay anchor.");
    }
    cui_end_frame(ui);
    return event;
}

/** Gets the rectangle of a rail button, including the first toggle. */
static cui_rect rail_button(cui_ctx *ui, const char *name, const char *tooltip, bool toggle) {
    cui_push_id(ui, "rail");
    if (toggle) cui_push_id_int(ui, 0);
    if (tooltip) cui_push_id(ui, tooltip);
    const cui_rect rect = cui_get_rect(ui, cui_make_id(ui, name));
    if (tooltip) cui_pop_id(ui);
    if (toggle) cui_pop_id(ui);
    cui_pop_id(ui);
    return rect;
}

/** Clicks a measured rail button and combines its press and release flags. */
static int rail_click(cui_ctx *ui, rail_case *sample, const char *name, const char *tooltip, bool toggle) {
    const cui_rect rect = rail_button(ui, name, tooltip, toggle);
    check(rect.w > 0 && rect.h > 0, "The requested rail button must have bounds.");
    cui_input_mouse_move(ui, rect.x + rect.w / 2, rect.y + rect.h / 2);
    rail_frame(ui, sample);
    click(ui, rect.x + rect.w / 2, rect.y + rect.h / 2, true);
    const int pressed = rail_frame(ui, sample);
    click(ui, rect.x + rect.w / 2, rect.y + rect.h / 2, false);
    return pressed | rail_frame(ui, sample);
}

/** Activates the focused rail button through the CUI keyboard path. */
static int rail_key(cui_ctx *ui, rail_case *sample, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
    return rail_frame(ui, sample);
}

/** Checks the rail's interaction, layout options and model proposals. */
static void canvas_rail(void) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create canvas rail context."); return; }
    rail_case sample = { .view = { 10, 6, 1 }, .flags = 1 };
    for (int i = 0; i < 4; ++i) rail_frame(ui, &sample);

    check(rail_click(ui, &sample, CUI_ICON_PLUS, "Zoom in", false) == CUI_CANVAS_ZOOMED
        && sample.view.zoom == 1.25f, "Default zoom in must select the next preset.");
    rail_key(ui, &sample, CUI_KEY_TAB);
    check(rail_key(ui, &sample, CUI_KEY_SPACE) == CUI_CANVAS_ZOOMED && sample.view.zoom == 1,
        "Tab from plus must reach the readout and Space must restore 100 percent.");

    static const float high[] = { 1.25f, 1.5f, 2 };
    sample.steps = high;
    sample.step_count = 3;
    sample.view = (cui_node_view){ 30, 40, .8f };
    check(rail_frame(ui, &sample) == 0 && sample.view.zoom == .8f
        && sample.view.x == 30 && sample.view.y == 40, "Custom presets must leave an idle external viewport unchanged.");
    check(rail_click(ui, &sample, CUI_ICON_PLUS, "Zoom in", false) == CUI_CANVAS_ZOOMED
        && sample.view.zoom == 1.25f
        && fabsf((300 - sample.view.x) / sample.view.zoom - (300 - 30) / .8f) < .001f,
        "Zooming into the preset range must preserve the canvas center.");
    check(rail_click(ui, &sample, "#zoom", NULL, false) == CUI_CANVAS_ZOOMED
        && sample.view.zoom == 1, "Reset must reach 100 percent outside the custom preset range.");

    static const float close[] = { 1, 1.0005f, 1.002f };
    sample.steps = close;
    check(rail_click(ui, &sample, CUI_ICON_PLUS, "Zoom in", false) == CUI_CANVAS_ZOOMED
        && sample.view.zoom == close[1], "Zoom in must visit a close custom preset.");
    check(rail_click(ui, &sample, CUI_ICON_MINUS, "Zoom out", false) == CUI_CANVAS_ZOOMED
        && sample.view.zoom == close[0], "Zoom out must visit a close custom preset.");

    sample.steps = NULL;
    sample.step_count = 0;
    sample.bounds = (cui_rect){ -200, -40, 360, 400 };
    check(rail_click(ui, &sample, "", "Fit", false) == (CUI_CANVAS_FIT | CUI_CANVAS_ZOOMED)
        && fabsf(sample.view.zoom - .72f) < .001f && fabsf(sample.view.x - 314.4f) < .001f
        && fabsf(sample.view.y - 44.8f) < .001f, "Fit must center the supplied canvas bounds at the fitting zoom.");
    sample.bounds = (cui_rect){ 10, 20, 1, 1 };
    check(rail_click(ui, &sample, "", "Fit", false) == (CUI_CANVAS_FIT | CUI_CANVAS_ZOOMED)
        && sample.view.zoom == 4 && sample.view.x == 258 && sample.view.y == 78,
        "Fit must limit its zoom to the preset range and center the bounds at that zoom.");
    sample.bounds = (cui_rect){ 0 };
    const cui_node_view before = sample.view;
    check(rail_click(ui, &sample, "", "Fit", false) == CUI_CANVAS_FIT && sample.view.zoom == before.zoom
        && sample.view.x == before.x && sample.view.y == before.y, "Fit with empty bounds must only report the request.");
    check(rail_click(ui, &sample, "#toggle", NULL, true) == CUI_CANVAS_TOGGLED
        && sample.pressed, "A toggle can use the default icon and report the flipped state.");
    const cui_id toggle_focus = cui_focused_id(ui);
    sample.toggle = (fpas_canvas_toggle){ CUI_ICON_LAYERS, "Hide layers" };
    rail_frame(ui, &sample);
    check(rail_key(ui, &sample, CUI_KEY_ENTER) == CUI_CANVAS_TOGGLED
        && !sample.pressed && cui_focused_id(ui) == toggle_focus,
        "Changing a toggle icon and tooltip must keep keyboard focus on the same toggle.");

    const cui_rect visible_fit = cui_get_rect(ui, cui_make_id(ui, "rail"));
    sample.flags = 1 | 2 | 4;
    for (int i = 0; i < 3; ++i) rail_frame(ui, &sample);
    const cui_rect anchor = cui_get_rect(ui, cui_make_id(ui, "rail-canvas"));
    const cui_rect start = cui_get_rect(ui, cui_make_id(ui, "rail"));
    check(fabsf(start.x - anchor.x - 12) < .5f, "Start-side places the rail on the left in LTR.");
    check(fabsf(start.h - visible_fit.h + 38) < .5f, "HideFit must remove the Fit button.");
    sample.rtl = true;
    for (int i = 0; i < 3; ++i) rail_frame(ui, &sample);
    const cui_rect rtl_anchor = cui_get_rect(ui, cui_make_id(ui, "rail-canvas"));
    const cui_rect rtl = cui_get_rect(ui, cui_make_id(ui, "rail"));
    check(fabsf(rtl_anchor.x + rtl_anchor.w - rtl.x - rtl.w - 12) < .5f,
        "Start-side places the rail on the right in RTL.");

    check(rail_click(ui, &sample, "#zoom", NULL, false) == CUI_CANVAS_ZOOMED, "Reset must accept pointer input.");
    check(rail_key(ui, &sample, CUI_KEY_ENTER) == 0, "Enter at 100 percent must leave the viewport unchanged.");
    sample.view.zoom = 1.5f;
    sample.popup = true;
    rail_frame(ui, &sample);
    check(rail_key(ui, &sample, CUI_KEY_ENTER) == 0 && sample.view.zoom == 1.5f,
        "A covered rail button must leave Enter to the popover.");
    check(rail_key(ui, &sample, CUI_KEY_SPACE) == 0 && sample.view.zoom == 1.5f,
        "A covered rail button must leave Space to the popover.");
    cui_popover_close(ui, "Options");
    sample.popup = false;
    rail_frame(ui, &sample);

    cui_set_reduced_motion(ui, false);
    check(rail_click(ui, &sample, CUI_ICON_PLUS, "Zoom in", false) == CUI_CANVAS_ZOOMED && sample.view.zoom > 1.5f
        && sample.view.zoom < 2, "With motion enabled, zoom must ease toward its preset.");
    const float easing = sample.view.zoom;
    sample.flags &= ~1;
    check(rail_frame(ui, &sample) == 0 && sample.view.zoom == easing,
        "Disabling the rail must stop pending viewport changes.");
    check(rail_click(ui, &sample, "#toggle", NULL, true) == 0 && !sample.pressed,
        "A disabled toggle must leave its state unchanged.");
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
            int next = fpas_cui_toolbar_cycle(ui, "type", types, 3, current, "Format", NULL, CUI_TONE_NEUTRAL, 0);
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

/** Draws the presence pill and returns the current vertex count. */
static int presence_frame(cui_ctx *ui, int count) {
    const cui_live_user people[] = {{ .name = "Ari" }, { .name = "Emi", .away = true }, { .name = "Yuki" }};
    cui_begin_frame(ui, .2f);
    cui_column(ui, .pad = 60) {
        fpas_cui_presence_pill(ui, "pill", people, NULL, count, 30, 2);
    }
    cui_end_frame(ui);
    return cui_get_draw_data(ui)->vertex_count;
}

/** Checks that only the hovered visible avatar schedules and shows a tooltip. */
static void presence_tooltips(void) {
    cui_ctx *ui = canvas();
    if (!ui) { check(false, "Cannot create presence context."); return; }

    cui_input_mouse_move(ui, 350, 240);
    for (int k = 0; k < 5; ++k) {
        presence_frame(ui, 1);
        check(cui_next_wakeup(ui) < 0, "A single avatar must stay idle outside the pill.");
    }
    const int idle = presence_frame(ui, 3);
    check(cui_next_wakeup(ui) < 0, "Several avatars must stay idle outside the pill.");

    const cui_rect pill = cui_get_rect(ui, cui_make_id(ui, "pill"));
    cui_input_mouse_move(ui, pill.x + 32, pill.y + pill.h * .5f);
    check(presence_frame(ui, 3) == idle && cui_next_wakeup(ui) > 0,
        "Hovering the overlapping avatars must schedule a tooltip.");
    int hovered = idle;
    for (int k = 0; k < 5; ++k) hovered = presence_frame(ui, 3);
    const cui_rect tip = cui_get_rect(ui, 0x7001718u);
    check(hovered > idle && tip.w > 60 && cui_next_wakeup(ui) < 0,
        "Hover must show the topmost, away avatar's tooltip and stop scheduling wakeups.");

    cui_input_mouse_leave(ui);
    check(presence_frame(ui, 3) == idle && cui_next_wakeup(ui) < 0,
        "Leaving must hide the tooltip and stop the hover timer.");
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
    graph_reports_status(false);
    graph_reports_status(true);
    graph_pan_release();
    graph_selection_ring();
    format_and_cycle();
    canvas_rail();
    presence_tooltips();
    return failures != 0;
}
