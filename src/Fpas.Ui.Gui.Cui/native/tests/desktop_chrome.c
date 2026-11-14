/* Native input, saved state and exact rendering parity of desktop chrome. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int failures;
#define CHECK(test) do { if (!(test)) { fprintf(stderr, "%d: %s\n", __LINE__, #test); ++failures; } } while (0)

static void key(cui_ctx *ui, cui_key value, int mods) {
    cui_input_key(ui, value, true, mods);
    cui_input_key(ui, value, false, mods);
}

static void tabs(cui_ctx *ui, bool enabled, bool focus, int *selected, int *close,
    int *from, int *to, int *nodes) {
    const char *titles[] = { "README.md", "main.c", "main.h" };
    const char *icons[] = { CUI_ICON_FILE_TEXT, CUI_ICON_CODE, CUI_ICON_CODE };
    const char *tips[] = { "", "src/main.c", "" };
    const int dirty[] = { 0, 1, 0 };
    fpas_cui_document_tabs(ui, "docs", titles, icons, tips, dirty, 3, selected,
        1 | (enabled ? 8 : 0) | (focus ? 16 : 0), close, from, to, nodes);
}

static void documents(cui_ctx *ui) {
    int selected = 0, close, from, to, nodes[4];
    for (int frame = 0; frame <= 6; ++frame) {
        if (frame == 1) key(ui, CUI_KEY_RIGHT, 0);
        if (frame == 2) key(ui, CUI_KEY_LEFT, CUI_MOD_SHIFT);
        if (frame == 3 || frame == 5) key(ui, CUI_KEY_W, CUI_MOD_CTRL);
        cui_begin_frame(ui, .016f);
        tabs(ui, frame != 5, frame == 0, &selected, &close, &from, &to, nodes);
        cui_end_frame(ui);
        if (frame == 1) CHECK(selected == 1);
        if (frame == 2) CHECK(from == 1 && to == 0 && selected == 0);
        if (frame == 3) CHECK(close == 0);
        if (frame == 5) CHECK(close == -1 && selected == 0);
    }
}

static void operations(cui_ctx *ui) {
    const char *keys[] = { "wrap" }, *labels[] = { "Word wrap" }, *empty[] = { "" };
    const int flags[] = { 16 };
    int shown = 0, chosen = -1, item = -1, nodes[2], open, highlight;
    for (int frame = 0; frame < 10; ++frame) {
        if (frame == 2) key(ui, CUI_KEY_F10, 0);
        if (frame == 3) key(ui, CUI_KEY_DOWN, 0);
        if (frame == 4) key(ui, CUI_KEY_ENTER, 0);
        if (frame == 6) key(ui, CUI_KEY_SPACE, 0);
        cui_begin_frame(ui, .016f);
        fpas_cui_menu_bar_begin(ui, "bar", 1, 1);
        int result = fpas_cui_menu_bar_menu(ui, "View", 0, 0, 1, keys, labels, empty,
            empty, flags, 1, nodes, &open, &highlight);
        if (open) ++shown;
        if (result >= 0) chosen = result;
        fpas_cui_menu_bar_end(ui);
        fpas_cui_status_bar_begin(ui, "status", 1);
        fpas_cui_status_bar_text(ui, "Saved", CUI_ICON_CHECK, CUI_TONE_SUCCESS);
        fpas_cui_status_bar_spacer(ui);
        if (fpas_cui_status_bar_item(ui, "UTF-8###encoding", NULL, 0, NULL, 1, frame == 5, &item))
            CHECK(frame == 6);
        else if (frame == 6) CHECK(false);
        fpas_cui_status_bar_end(ui);
        cui_end_frame(ui);
    }
    CHECK(shown > 0 && chosen == 0 && item >= 0);
}

static void split(cui_ctx *ui) {
    float ratio = .35f;
    for (int frame = 0; frame < 9; ++frame) {
        const cui_rect bounds = cui_get_rect(ui, cui_make_id(ui, "split"));
        if (frame == 3) cui_input_mouse_move(ui, bounds.x + (bounds.w - 1) * ratio, bounds.y + 25);
        if (frame == 4) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 5) cui_input_mouse_move(ui, bounds.x + bounds.w * .65f, bounds.y + 25);
        if (frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        ratio = fpas_cui_split_view_begin(ui, "split", ratio, 0, 96, 120, 2, 480, 2, 160, 1);
        cui_text(ui, "Notes");
        fpas_cui_split_view_next(ui, 1);
        cui_text(ui, "Selected note");
        fpas_cui_split_view_end(ui);
        cui_end_frame(ui);
    }
    CHECK(fabsf(ratio - .65f) < .002f);
}

static bool narrow_split(cui_ctx *ui, bool bridge) {
    float ratio = .5f;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 3) cui_input_mouse_move(ui, bridge ? 50 : 120, 25);
        if (frame == 4) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame == 5) cui_input_mouse_move(ui, 80, 25);
        if (frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        if (bridge)
            ratio = fpas_cui_split_view_begin(ui, "tiny", ratio, 0, 0, 0, 2, 100, 2, 120, 1);
        else
            cui_split_begin(ui, "tiny", &ratio, &(cui_split_opts){ .width = CUI_PX(100), .height = CUI_PX(120) });
        cui_text(ui, "Left");
        if (bridge) fpas_cui_split_view_next(ui, 1); else cui_split_next(ui);
        cui_text(ui, "Right");
        if (bridge) fpas_cui_split_view_end(ui); else cui_split_end(ui);
        cui_end_frame(ui);
    }
    printf("%s narrow split ratio: %g\n", bridge ? "FPAS" : "CUI", ratio);
    return ratio >= .49f && ratio <= .51f;
}

static void dock(cui_ctx *ui) {
    const char *saved = "dock1 h0.35([*Scene tree],[*Viewport,-Log])";
    char before[256], after[256];
    int open = 1, tabs[3] = {0}, visible = 0;
    CHECK(fpas_cui_dockspace_load(ui, "workspace", saved));
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 4) open = 1;
        cui_begin_frame(ui, .016f);
        fpas_cui_dockspace_begin(ui, "workspace", NULL, 1, 2, 740, 2, 220, 0, 0, 1);
        if (fpas_cui_dock_panel_begin(ui, "Scene tree", CUI_ICON_LAYERS, 0, 0, &tabs[0], &tabs[1])) {
            cui_text(ui, "harbor_town");
            fpas_cui_dock_panel_end(ui);
        }
        if (fpas_cui_dock_panel_begin(ui, "Viewport", CUI_ICON_BOX, 0, 0, &tabs[0], &tabs[1])) {
            cui_text(ui, "Perspective");
            fpas_cui_dock_panel_end(ui);
        }
        if (fpas_cui_dock_panel_begin(ui, "Log", CUI_ICON_SQUARE_TERMINAL, 0, 1, &open, &tabs[2])) {
            ++visible;
            cui_text(ui, "Build finished");
            fpas_cui_dock_panel_end(ui);
        }
        fpas_cui_dockspace_end(ui);
        cui_end_frame(ui);
        if (frame < 4) CHECK(!open);
    }
    CHECK(open && visible && tabs[2] >= 0);
    CHECK(fpas_cui_dockspace_save(ui, "workspace", before, sizeof before) == strlen(before));
    CHECK(strstr(before, "*Log") != NULL);
    CHECK(!fpas_cui_dockspace_load(ui, "workspace", "dock1 nonsense"));
    fpas_cui_dockspace_save(ui, "workspace", after, sizeof after);
    CHECK(strcmp(before, after) == 0);
}

static void dock_drag(cui_ctx *ui) {
    const char *titles[] = { "Scene tree", "Viewport", "Log" };
    int nodes[3] = { -1, -1, -1 };
    cui_rect tab = { 0 }, target = { 0 };
    char initial[256] = "", moved[256];
    for (int frame = 0; frame < 22; ++frame) {
        if (frame == 5) cui_input_mouse_move(ui, tab.x + 20, tab.y + tab.h * .5f);
        if (frame == 6) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        if (frame >= 7 && frame < 17)
            cui_input_mouse_move(ui, target.x + target.w * .88f, target.y + 45);
        if (frame == 17) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        cui_begin_frame(ui, .016f);
        fpas_cui_dockspace_begin(ui, "workspace", "dock1 h0.24([*Scene tree],v0.65([*Viewport],[*Log]))",
            0, 2, 740, 2, 320, 0, 0, 1);
        for (int index = 0; index < 3; ++index) {
            int open = 1;
            if (fpas_cui_dock_panel_begin(ui, titles[index], NULL, 0, 1, &open, &nodes[index])) {
                cui_text(ui, titles[index]);
                fpas_cui_dock_panel_end(ui);
            }
        }
        fpas_cui_dockspace_end(ui);
        cui_end_frame(ui);
        if (frame == 4) {
            tab = fpas_cui_node_rect(ui, nodes[2]);
            target = fpas_cui_node_rect(ui, nodes[1]);
            fpas_cui_dockspace_save(ui, "workspace", initial, sizeof initial);
        }
    }
    CHECK(tab.w > 0 && target.w > 0);
    CHECK(fpas_cui_dockspace_save(ui, "workspace", moved, sizeof moved) > 0);
    CHECK(strcmp(initial, moved) != 0 && strstr(moved, "Log") != NULL);
}

static void annotation(cui_ctx *ui, bool bridge, bool marked) {
    const float samples[] = { 4, 10, 9, 16 };
    const int accent = 1;
    const cui_dot_series band = { samples, NULL, true };
    const cui_chart_annotation native[] = {
        { CUI_ANNOTATION_X, 16, 0, "BED", "16:00", true },
        { CUI_ANNOTATION_Y, 0, 10, "10" },
        { CUI_ANNOTATION_POINT, 12, 10, "PEAK", NULL, true }
    };
    const fpas_chart_annotation marks[] = {
        { CUI_ANNOTATION_X, 16, 0, "BED", "16:00", 1 },
        { CUI_ANNOTATION_Y, 0, 10, "10" },
        { CUI_ANNOTATION_POINT, 12, 10, "PEAK", NULL, 1 }
    };
    const int count = marked ? 3 : 0;
    if (bridge) {
        const fpas_dot_options opts = { .chart = { .height = 160, .minimum = 0, .maximum = 20,
            .annotations = marks, .annotation_count = count }, .x_min = 8, .x_max = 20, .flags = 4 };
        int index; double x; float value;
        fpas_cui_dot_chart(ui, "chart", samples, &accent, 1, 4, &opts, &index, &x, &value);
    } else
        cui_dot_chart(ui, "chart", &band, 1, 4, .height = 160, .min = 0, .max = 20,
            .x_min = 8, .x_max = 20, .annotations = native, .annotation_count = count);
}

static void chrome_pixels(cui_ctx *ui, bool bridge) {
    const char *keys[] = { "wrap" }, *labels[] = { "Word wrap" }, *empty[] = { "" };
    const int flags[] = { 16 };
    int selected = 1, close, from, to, nodes[4], open, highlighted, item;
    if (bridge) {
        fpas_cui_menu_bar_begin(ui, "bar", 1, 1);
        fpas_cui_menu_bar_menu(ui, "View###view", 0, 0, 1, keys, labels, empty, empty, flags, 1, nodes, &open, &highlighted);
        fpas_cui_menu_bar_end(ui);
        tabs(ui, true, false, &selected, &close, &from, &to, nodes);
        fpas_cui_status_bar_begin(ui, "status", 1);
        fpas_cui_status_bar_item(ui, "main###branch", CUI_ICON_GIT_BRANCH, 0, "Switch branch", 1, 0, &item);
        fpas_cui_status_bar_text(ui, "Saved", CUI_ICON_CHECK, CUI_TONE_SUCCESS);
        fpas_cui_status_bar_progress(ui, .62f, "Indexing 62%", 0, 0);
        fpas_cui_status_bar_spacer(ui);
        fpas_cui_status_bar_item(ui, "UTF-8###encoding", NULL, 0, NULL, 1, 0, &item);
        fpas_cui_status_bar_end(ui);
    } else {
        cui_menu_bar(ui, "bar") {
            cui_menu_bar_menu(ui, "View") { cui_menu_item(ui, "Word wrap", .checked = true); }
        }
        const cui_doc_tab docs[] = { { "README.md", CUI_ICON_FILE_TEXT },
            { "main.c", CUI_ICON_CODE, "src/main.c", true }, { "main.h", CUI_ICON_CODE } };
        cui_doc_tabs(ui, "docs", docs, 3, &selected, .shortcuts = true);
        cui_status_bar(ui, "status") {
            cui_status_bar_item(ui, "main", .icon = CUI_ICON_GIT_BRANCH, .tooltip = "Switch branch");
            cui_status_bar_text(ui, "Saved", .icon = CUI_ICON_CHECK, .tone = CUI_TONE_SUCCESS);
            cui_status_bar_progress(ui, .62f, .label = "Indexing 62%");
            cui_status_bar_spacer(ui);
            cui_status_bar_item(ui, "UTF-8");
        }
    }
}

static void pane_pixels(cui_ctx *ui, bool bridge) {
    float ratio = .35f;
    const cui_split_opts split = { .width = CUI_FILL, .height = CUI_PX(68), .min_first = 96, .min_second = 120 };
    if (bridge) fpas_cui_split_view_begin(ui, "split", ratio, 0, 96, 120, 1, 0, 2, 68, 1);
    else cui_split_begin(ui, "split", &ratio, &split);
    cui_text(ui, "Notes");
    if (bridge) fpas_cui_split_view_next(ui, 1); else cui_split_next(ui);
    cui_text(ui, "Selected note");
    if (bridge) fpas_cui_split_view_end(ui); else cui_split_end(ui);

    const char *layout = "dock1 h0.40([*Scene tree],[*Viewport,Log])";
    const char *titles[] = { "Scene tree", "Viewport", "Log" };
    const char *icons[] = { CUI_ICON_LAYERS, CUI_ICON_BOX, CUI_ICON_SQUARE_TERMINAL };
    if (bridge) fpas_cui_dockspace_begin(ui, "workspace", layout, 1, 1, 0, 2, 126, 0, 0, 1);
    else cui_dockspace_begin(ui, "workspace", .layout = layout, .height = CUI_PX(126), .shortcuts = true);
    for (int i = 0; i < 3; ++i) {
        int present = 1, node;
        bool open = true;
        const bool visible = bridge ? fpas_cui_dock_panel_begin(ui, titles[i], icons[i], 0, 1, &present, &node)
            : cui_dock_panel_begin(ui, titles[i], &open, .icon = icons[i]);
        if (!visible) continue;
        cui_text(ui, titles[i]);
        if (bridge) fpas_cui_dock_panel_end(ui); else cui_dock_panel_end(ui);
    }
    if (bridge) fpas_cui_dockspace_end(ui); else cui_dockspace_end(ui);
}

static void render(unsigned char *pixels, bool bridge, bool marked, bool dark, int width) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, width, 500, 1);
    cui_set_reduced_motion(ui, true);
    cui_set_theme(ui, dark ? CUI_THEME_DARK : CUI_THEME_LIGHT);
    cui_input_mouse_leave(ui);
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 5) key(ui, CUI_KEY_F10, 0);
        if (frame == 6) key(ui, CUI_KEY_DOWN, 0);
        cui_begin_frame(ui, .016f);
        chrome_pixels(ui, bridge);
        annotation(ui, bridge, marked);
        pane_pixels(ui, bridge);
        cui_end_frame(ui);
    }
    cui_sw_render(sw, cui_get_draw_data(ui), pixels, width, 500, width * 4, CUI_SW_RGBA);
    cui_destroy(ui);
    cui_sw_destroy(sw);
}

static void pixels(int width, bool dark) {
    const size_t count = (size_t)width * 500 * 4;
    unsigned char *actual = malloc(count), *expected = malloc(count), *unmarked = malloc(count);
    if (!actual || !expected || !unmarked) abort();
    render(actual, true, true, dark, width);
    render(expected, false, true, dark, width);
    render(unmarked, false, false, dark, width);
    CHECK(memcmp(actual, expected, count) == 0);
    CHECK(memcmp(expected, unmarked, count) != 0);
    free(actual);
    free(expected);
    free(unmarked);
}

static cui_ctx *context(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 800, 440, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

int main(int argc, char **argv) {
    cui_ctx *narrow = context();
    const bool bounded = narrow_split(narrow, argc < 2 || strcmp(argv[1], "native") != 0);
    cui_destroy(narrow);
    if (!bounded) return 1;
    void (*cases[])(cui_ctx *) = { documents, operations, split, dock, dock_drag };
    for (size_t i = 0; i < sizeof cases / sizeof *cases; ++i) {
        cui_ctx *ui = context();
        cases[i](ui);
        cui_destroy(ui);
    }
    for (int dark = 0; dark < 2; ++dark) {
        pixels(740, dark);
        pixels(360, dark);
    }
    return failures ? 1 : 0;
}
