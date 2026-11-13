#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>

static int failures;

static void check(bool pass, const char *label) {
    if (!pass) { fprintf(stderr, "%s\n", label); ++failures; }
}

static cui_ctx *context(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 750, 470, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void press(cui_ctx *ui, cui_key key, int mods) {
    cui_input_key(ui, key, true, mods);
    cui_input_key(ui, key, false, mods);
}

static void inspector(void) {
    cui_ctx *ui = context();
    fpas_property row = {.label = "Cast shadows", .kind = 3, .integer = 1, .flags = 1, .digits = -1};
    fpas_property_group group = {.title = "Shadows", .properties = &row, .count = 1, .open = 0, .collapsible = 1};
    int expansion = 0, edits = 0;
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2 || frame == 5) press(ui, CUI_KEY_TAB, 0);
        if (frame == 3 || frame == 6) press(ui, CUI_KEY_SPACE, 0);
        cui_begin_frame(ui, .016f);
        fpas_cui_property_inspector(ui, "props", &group, 1, 0, 1, 0, 1);
        expansion += group.changed;
        edits += row.changed;
        row.changed = 0;
        cui_end_frame(ui);
    }
    check(expansion == 1 && group.open, "Keyboard must open the inspector group once.");
    check(edits == 1 && !row.integer, "Switching a property must return its typed boolean proposal.");
    cui_destroy(ui);
}

static void flame(void) {
    cui_ctx *ui = context();
    const cui_flame_span spans[] = {{"Frame",0,1,0,0},{"Update",.10,.30,1,0},{"Render",.40,.75,1,0}};
    fpas_flame_view range = {.selected = 0};
    int zoom = -1;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2) press(ui, CUI_KEY_DOWN, 0);
        if (frame == 4) press(ui, CUI_KEY_ENTER, 0);
        cui_begin_frame(ui, .016f);
        if (fpas_cui_flame_graph(ui,"graph",spans,3,&range,0,180,0,1,0,frame == 0 ? 3 : 1)
            && range.zoomed >= 0) zoom = range.zoomed;
        cui_end_frame(ui);
    }
    check(range.selected == 1 && zoom == 1 && range.end - range.start < .9,
        "Down must select the first child and Enter must focus its time range.");
    cui_destroy(ui);
}

static void curve(void) {
    cui_ctx *ui = context();
    cui_curve_key keys[6] = {{0,.2f},{.50f,.5f},{1,.8f}};
    int count = 3, selected = -1, added, removed = -1, erased = -1;
    fpas_curve_options options = {.x_max = 1, .y_max = 1, .width_kind = 1, .flags = 1};
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2 || frame == 4) press(ui, CUI_KEY_TAB, 0);
        if (frame == 5) press(ui, CUI_KEY_UP, 0);
        if (frame == 7) press(ui, CUI_KEY_DELETE, 0);
        cui_begin_frame(ui, .016f);
        fpas_cui_curve_editor(ui, "curve", keys, &count, 6, &selected, &added, &removed, &options);
        if (frame == 5) check(selected == 1 && keys[1].y > .5f, "Arrow must nudge the selected curve key.");
        if (removed >= 0) erased = removed;
        cui_end_frame(ui);
    }
    check(count == 2 && erased == 1 && keys[1].x == 1, "Delete must remove only the selected curve key.");
    cui_destroy(ui);
}

static void gradient(void) {
    cui_ctx *ui = context();
    cui_gradient_stop stops[6] = {{0,{255,0,0,255}},{.5f,{0,255,0,128}},{1,{0,0,255,0}}};
    int count = 3, selected = 0, added, removed, erased = -1;
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2 || frame == 4) press(ui, CUI_KEY_TAB, 0);
        if (frame == 5) press(ui, CUI_KEY_RIGHT, 0);
        if (frame == 7) press(ui, CUI_KEY_DELETE, 0);
        cui_begin_frame(ui, .016f);
        fpas_cui_gradient_editor(ui,"ramp",stops,&count,6,&selected,&added,&removed,1,0,0,1);
        if (frame == 5)
            check(selected == 1 && stops[1].pos > .5f && stops[1].color.a == 128, "Nudging a stop must preserve its color and alpha.");
        if (removed >= 0) erased = removed;
        cui_end_frame(ui);
    }
    check(count == 2 && erased == 1 && stops[1].color.a == 0, "Delete must preserve the transparent endpoint.");
    cui_destroy(ui);
}

static void timeline(void) {
    cui_ctx *ui = context();
    cui_curve_key keys[6] = {{0,.2f},{1,.5f},{2,.8f}};
    bool selected[] = {false,true,false,false,false,false};
    cui_timeline_track track = {.name = "Scale", .keys = keys, .key_count = 3, .key_capacity = 6, .key_selected = selected};
    fpas_timeline_view range = {.time = 1, .in = .25f, .out = 1.75f, .track = 0, .key = 1, .range = 1};
    fpas_timeline_options options = {.end = 2, .fps = 24, .height = 160, .width_kind = 1, .flags = 11};
    int plays = 0;
    for (int frame = 0; frame < 11; ++frame) {
        if (frame == 2) press(ui, CUI_KEY_RIGHT, 0);
        if (frame == 4) press(ui, CUI_KEY_RIGHT, CUI_MOD_ALT);
        if (frame == 6) press(ui, CUI_KEY_SPACE, 0);
        if (frame == 8) press(ui, CUI_KEY_DELETE, 0);
        cui_begin_frame(ui, .016f);
        int result = fpas_cui_timeline_editor(ui, "shot", &track, 1, &range, &options);
        options.flags = 3;
        if (frame == 2) check((result & 1) && fabsf(range.time - 25.0f/24.0f) < .001f, "Timeline must advance exactly one frame.");
        if (frame == 4) check((result & 1) && fabsf(keys[1].x - 25.0f/24.0f) < .001f, "Alt+Right must move the selected key one frame.");
        plays += (result & 2) != 0;
        cui_end_frame(ui);
    }
    check(plays == 1 && track.key_count == 2 && keys[1].x == 2, "Timeline must request playback once and delete only the selected key.");
    cui_destroy(ui);
}

static void disabled_wheel(bool native) {
    for (int widget = 0; widget < 2; ++widget) {
        cui_ctx *ui = context();
        const cui_flame_span spans[] = {{"Frame",0,2,0,0},{"Update",.2,1.5,1,0}};
        fpas_flame_view flame = {.start = 0, .end = 2, .selected = -1};
        fpas_timeline_view timeline = {.start = 0, .end = 2, .track = -1, .key = -1};
        fpas_timeline_options options = {.end = 2, .fps = 24, .height = 170, .width_kind = 1};
        cui_timeline_track track = {.name = "Scale"};
        for (int frame = 0; frame < 6; ++frame) {
            if (frame == 2) cui_input_mouse_move(ui, 460, 95);
            if (frame == 3) cui_input_scroll(ui, 0, 1);
            cui_begin_frame(ui, .016f);
            if (!native) {
                if (widget == 0) fpas_cui_flame_graph(ui, "disabled", spans, 2, &flame, 0, 170, 0, 1, 0, 0);
                else fpas_cui_timeline_editor(ui, "disabled", &track, 1, &timeline, &options);
            } else {
                cui_push_disabled(ui, true);
                if (widget == 0) {
                    cui_flame_view view = {flame.start, flame.end, flame.scroll};
                    cui_flame_graph(ui, "disabled", spans, 2, &view, NULL, .height = 170);
                    flame.start = view.start;
                    flame.end = view.end;
                } else {
                    cui_timeline_view view = {timeline.start, timeline.end, timeline.scroll};
                    cui_timeline_editor(ui, "disabled", &track, 1, &timeline.time, .end = 2, .fps = 24, .height = 170, .view = &view);
                    timeline.start = view.start;
                    timeline.end = view.end;
                }
                cui_pop_disabled(ui);
            }
            cui_end_frame(ui);
        }
        check(widget == 0 ? flame.end - flame.start == 2 : timeline.end - timeline.start == 2,
            widget == 0 ? "Disabled flame graph zoomed on wheel input." : "Disabled timeline zoomed on wheel input.");
        cui_destroy(ui);
    }
}

int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) {
        disabled_wheel(true);
        return failures ? 1 : 0;
    }
    disabled_wheel(false);
    inspector();
    flame();
    curve();
    gradient();
    timeline();
    return failures ? 1 : 0;
}
