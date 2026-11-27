// [Documentation](../../../../docs/ui/gui/developer-tools/dependency-graph.md)
// [Commit graph](../../../../docs/ui/gui/developer-tools/commit-graph.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static const cui_graph_node vertices[] = {
    { "App", "Executable", CUI_TONE_VIOLET }, { "Library", NULL, CUI_TONE_INFO }
};
static const cui_graph_edge edges[] = { { 0, 1 } };
static const cui_commit commits[] = {
    { "0123456789", "Merge\nDetails", "Author", "Today", "main, tag: v1", { 1, -1 } },
    { "fedcba9876", "Initial", "Author", "Yesterday", NULL, { -1, -1 } }
};

/** Sends one complete key press. */
static void tap(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

/** Checks retained selection, explicit focus, popup ownership and subsequent activation. */
static bool keyboard_scope(bool commit) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 740, 380, 1);
    cui_set_reduced_motion(ui, true);
    const cui_key ahead = commit ? CUI_KEY_DOWN : CUI_KEY_RIGHT;
    const cui_key back = commit ? CUI_KEY_UP : CUI_KEY_LEFT;
    const cui_key key[] = { CUI_KEY_NONE, ahead, ahead, CUI_KEY_NONE, back,
        CUI_KEY_ENTER, CUI_KEY_ESCAPE, CUI_KEY_ENTER };
    const int selected[] = { -1, 0, 1, 1, 1, 1, 1, 1 };
    bool valid = true;
    for (int frame = 0; frame < (int)(sizeof selected / sizeof *selected); ++frame) {
        if (key[frame] != CUI_KEY_NONE) tap(ui, key[frame]);
        cui_begin_frame(ui, .016f);
        cui_button(ui, "Popover anchor");
        if (frame == 3) cui_popover_open(ui, "Options");
        cui_popover(ui, "Options", .width = 150) { cui_text(ui, "Overlay"); }
        const int flags = frame == 0 ? 3 : 1;
        fpas_dev_result result;
        if (commit) fpas_cui_commit_graph(ui, "graph", commits, 2, -2, -1, 1, 0, flags, &result);
        else fpas_cui_dependency_graph(ui, "graph", vertices, 2, edges, 1, -2, 240, 0, 1, 0, flags, &result);
        const bool covered_keys_available = (frame != 4 && frame != 5) || cui_key_pressed(ui, key[frame]);
        cui_end_frame(ui);
        if (result.index != selected[frame] || result.action != (frame == 7 ? 1 : -1) ||
            result.flags != (frame == 1 || frame == 2) || !covered_keys_available ||
            !fpas_cui_focus_key(ui, cui_make_id(ui, "graph"))) {
            fprintf(stderr, "Graph keyboard frame %d (commit %d): index %d, activation %d, changed %d\n",
                frame, commit, result.index, result.action, result.flags);
            valid = false;
        }
    }
    fpas_cui_forget(ui, cui_make_id(ui, "graph"));
    valid = valid && cui_focused_id(ui) == 0;
    cui_destroy(ui);
    return valid;
}

/** Checks selection and hover after a disabled release and a subsequent normal click. */
static bool dependency_click(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 740, 380, 1);
    cui_set_reduced_motion(ui, true);
    const cui_id graph = cui_make_id(ui, "graph");
    bool valid = true;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2 || frame == 5) {
            const cui_rect bounds = cui_get_rect(ui, graph);
            cui_input_mouse_move(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3 || frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 7) cui_input_mouse_move(ui, 10, 320);
        cui_begin_frame(ui, .016f);
        fpas_dev_result result;
        cui_push_disabled(ui, frame == 3);
        fpas_cui_dependency_graph(ui, "graph", vertices, 1, NULL, 0, -2, 240, 0, 1, 0, 1, &result);
        cui_pop_disabled(ui);
        cui_end_frame(ui);
        const int hover = frame == 2 || (frame >= 4 && frame <= 6) ? 0 : -1;
        if (result.index != (frame >= 6 ? 0 : -1) || result.count != (frame == 6 ? 0 : -1) ||
            result.hover != hover || result.flags != (frame == 6)) {
            fprintf(stderr, "Dependency click frame %d: selection %d, click %d, hover %d\n",
                frame, result.index, result.count, result.hover);
            valid = false;
        }
    }
    cui_destroy(ui);
    return valid;
}

/** Compares commit widths and both dependency directions with native software pixels. */
static bool graph_pixels(bool commit, bool alternate) {
    enum { w = 740, h = 380, bytes = w * h * 4 };
    static uint8_t pixels[2][bytes];
    bool valid = true;
    const float width = alternate ? 410 : 680;
    for (int adapter = 0; adapter < 2; ++adapter) {
        cui_sw_renderer *renderer = cui_sw_create();
        if (!renderer) return false;
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, w, h, 1);
        cui_set_reduced_motion(ui, true);
        for (int frame = 0; frame < 6; ++frame) {
            cui_begin_frame(ui, .016f);
            if (frame >= 3) {
                fpas_dev_result result;
                int selected = 1;
                if (commit) {
                    if (adapter) fpas_cui_commit_graph(ui, "graph", commits, 2, selected, -1, 2, width, 1, &result);
                    else cui_commit_graph(ui, "graph", commits, 2, .selected = &selected, .width = CUI_PX(width), .height = -1);
                } else {
                    if (adapter) fpas_cui_dependency_graph(ui, "graph", vertices, 2, edges, 1, selected, 310, alternate, 2, width, 1, &result);
                    else cui_graph_view(ui, "graph", vertices, 2, edges, 1,
                        .selected = &selected, .width = CUI_PX(width), .height = 310, .top_down = alternate);
                }
            }
            cui_end_frame(ui);
            if (commit && alternate && adapter && frame == 3) valid = valid && cui_needs_redraw(ui);
            if (commit && alternate && adapter && frame == 4) valid = valid && !cui_needs_redraw(ui);
        }
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[adapter], w, h, w * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    return valid && memcmp(pixels[0], pixels[1], bytes) == 0;
}

/** Runs the developer graph regressions. */
int main(void) {
    const bool commit_keys = keyboard_scope(true), graph_keys = keyboard_scope(false);
    const bool click = dependency_click();
    bool pixels = true;
    for (int commit = 0; commit < 2; ++commit)
        for (int alternate = 0; alternate < 2; ++alternate)
            if (!graph_pixels(commit != 0, alternate != 0)) {
                fprintf(stderr, "Developer graph pixels or width settling differ (commit %d, alternate %d).\n", commit, alternate);
                pixels = false;
            }
    return commit_keys && graph_keys && click && pixels ? 0 : 1;
}
