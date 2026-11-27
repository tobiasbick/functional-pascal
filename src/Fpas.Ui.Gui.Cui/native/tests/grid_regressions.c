// [Documentation](../../../../docs/ui/gui/data-grid/README.md)
/* Direct CUI reproductions and focused checks of the adapters. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static bool grid_shrink(bool adapter) {
    const cui_grid_column column = { .title = "Name", .width = 200 };
    const char *titles[] = { column.title };
    const float widths[] = { column.width };
    const int flags[] = { 0 };
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 400, 260, 1);
    cui_set_reduced_motion(ui, true);
    int range[3], header, state[] = { -2, 0, -1, -1 };
    const cui_id scroll = cui_make_id(ui, "grid") ^ 0x5C011u;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 100, 140);
            cui_input_scroll(ui, 0, -60);
        }
        const int count = frame == 7 ? 1 : 100;
        cui_begin_frame(ui, .016f);
        if (adapter) {
            void *grid = fpas_cui_data_grid_begin(ui, "grid", titles, widths, flags, 1, count,
                NULL, state, 2, 160, 40, 1, 0, 1, range, &header);
            for (int row = range[0]; row < range[1]; ++row) {
                fpas_cui_data_grid_row(ui, row, "row", NULL);
                fpas_cui_data_grid_cell(ui, "Example");
            }
            fpas_cui_data_grid_end(ui, grid, NULL, state);
        } else {
            cui_data_grid_begin(ui, "grid", &column, 1, count, &range[0], &range[1],
                .height = CUI_PX(160));
            for (int row = range[0]; row < range[1]; ++row) {
                cui_data_grid_row(ui, row);
                cui_data_grid_cell_text(ui, "Example");
            }
            cui_data_grid_end(ui);
        }
        cui_end_frame(ui);
        if (frame == 6 && (range[0] <= 0 || cui_get_rect(ui, scroll).h < 100)) {
            fprintf(stderr, "The grid did not scroll before shrinking.\n");
            cui_destroy(ui);
            return false;
        }
    }
    printf("%s grid range after shrinking [%d,%d)\n", adapter ? "FPAS" : "CUI", range[0], range[1]);
    cui_destroy(ui);
    return range[0] == 0 && range[1] == 1;
}

static bool filter_add(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 360, 1);
    cui_set_reduced_motion(ui, true);
    const char *names[] = { "Team" }, *icons[] = { NULL }, *values[] = { "Engine" };
    const int counts[] = { 1 };
    const cui_filter_field field = { names[0], NULL, values, 1 };
    uint32_t selected = 0;
    char search[64] = "";
    int parts[5];
    cui_push_id(ui, "filters");
    cui_push_id_int(ui, 0);
    const cui_id chip = cui_make_id(ui, "chip");
    cui_pop_id(ui);
    cui_pop_id(ui);
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 1 || frame == 2) press(ui, CUI_KEY_TAB);
        if (frame == 3) press(ui, CUI_KEY_ENTER);
        if (frame == 5) press(ui, CUI_KEY_DOWN);
        if (frame == 6) press(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .016f);
        if (adapter)
            fpas_cui_filter_bar(ui, "filters", names, icons, values, counts, 1,
                &selected, search, sizeof search, 1, 0, parts);
        else
            cui_filter_bar(ui, "filters", &field, 1, &selected, search, sizeof search);
        cui_end_frame(ui);
    }
    const bool has_chip = cui_get_rect(ui, chip).w > 0;
    /* Query the popup by reproducing its seeded identity through a native rect. */
    const cui_rect panel = cui_get_rect(ui, chip ^ 0x3E40u);
    printf("%s added chip=%d options=%d\n",
        adapter ? "FPAS" : "CUI", has_chip, panel.w > 0);
    cui_destroy(ui);
    return has_chip && panel.w > 0;
}

/** Keeps native glyph spacing and the final mask bit in a focused, searchless filter bar. */
static bool filter_options(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 360, 1);
    cui_set_reduced_motion(ui, true);
    const char *names[] = { "Team" }, *icons[] = { NULL }, *values[32];
    for (int k = 0; k < 32; ++k) values[k] = "Engine";
    const int counts[] = { 32 };
    uint32_t selected = UINT32_C(0x80000000);
    int parts[5];
    float width[2];
    bool valid = true;
    for (int frame = 0; frame < 2; ++frame) {
        if (frame == 1) icons[0] = "";
        cui_begin_frame(ui, .016f);
        valid &= !fpas_cui_filter_bar(ui, "filters", names, icons, values, counts, 1,
            &selected, NULL, 0, 1, frame == 0, parts);
        cui_end_frame(ui);
        valid &= parts[1] < 0 && parts[2] < 0 && parts[3] >= 0
            && parts[4] >= 0 && fpas_cui_focused(ui, parts[4]);
        width[frame] = fpas_cui_node_rect(ui, parts[4]).w;
    }
    valid &= selected == UINT32_C(0x80000000) && width[0] > 0 && width[1] > width[0];
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Filter glyph geometry, high mask bit or fallback focus mismatch.\n");
    return valid;
}

static bool tree_lazy_load(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 360, 1);
    cui_set_reduced_motion(ui, true);
    const char *titles[] = { "Name", "Size" }, *labels[] = { "src", "button.c" };
    const float widths[] = { 240, 100 };
    const int column_flags[] = { 0, 2 }, parents[] = { -1, 0 };
    uint32_t flags[] = { CUI_TREE_HAS_CHILDREN, 0 };
    int state[] = { -2, 0, 0 }, events[2], load[64];
    bool valid = true;
    for (int frame = 0; frame < 5; ++frame) {
        if (frame == 1 || frame == 3) press(ui, CUI_KEY_RIGHT);
        if (frame == 4) press(ui, CUI_KEY_ENTER);
        int root, part, visible = 0;
        cui_begin_frame(ui, .016f);
        if (frame == 0) fpas_cui_request_focus(ui, "tree");
        if (frame == 2) flags[0] &= ~(uint32_t)CUI_TREE_LOADING;
        void *tree = fpas_cui_tree_table_begin(ui, "tree", titles, widths, column_flags, 2,
            labels, parents, flags, frame < 2 ? 1 : 2, state, 2, 160, 32, 3, &root);
        for (int row; (row = fpas_cui_tree_table_row(ui, &part)) >= 0;) {
            fpas_cui_tree_table_name(ui, labels[row], NULL);
            fpas_cui_data_grid_cell(ui, row == 0 ? "1.2 MB" : "24 KB");
            ++visible;
        }
        fpas_cui_tree_table_end(ui, tree, flags, state, events, load);
        cui_end_frame(ui);
        if (frame == 1)
            valid &= visible == 1 && events[1] == 1 && load[0] == 0 &&
                (flags[0] & CUI_TREE_LOADING) != 0;
        if (frame == 2) valid &= visible == 2 && events[1] == 0;
        if (frame == 3) valid &= state[2] == 1 && (flags[1] & CUI_TREE_SELECTED) != 0;
        if (frame == 4) valid &= events[0] == 1;
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Tree table lazy-load or keyboard navigation failed.\n");
    return valid;
}

static bool draw_tree_branch(cui_ctx *ui, int *open, int enabled, int *part) {
    const cui_table_column column = { .title = "Name", .width = CUI_FILL };
    cui_begin_frame(ui, .016f);
    cui_table_begin_ex(ui, "tree-cells", &column, 1, NULL);
    cui_table_row(ui, false);
    cui_table_cell(ui);
    const bool toggled = fpas_cui_tree_cell(ui, "src-name", "src", CUI_ICON_FOLDER, 0, open, enabled, part);
    cui_table_end(ui);
    cui_end_frame(ui);
    return toggled;
}

/** A press ends when the model removes a chevron, even if the same cell remains. */
static bool tree_cell_releases_chevron(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 440, 200, 1);
    cui_set_reduced_motion(ui, true);
    int open = 0, part = 0;
    draw_tree_branch(ui, &open, 1, &part);
    const cui_rect chevron = fpas_cui_node_rect(ui, part);
    bool valid = chevron.w == 20 && chevron.h == 20;
    cui_input_mouse_move(ui, chevron.x + chevron.w / 2, chevron.y + chevron.h / 2);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_tree_branch(ui, &open, 1, &part);
    open = -1;
    draw_tree_branch(ui, &open, 1, &part);
    valid &= open == -1 && fpas_cui_node_rect(ui, part).w > chevron.w;
    open = 0;
    draw_tree_branch(ui, &open, 1, &part);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    valid &= !draw_tree_branch(ui, &open, 1, &part) && open == 0;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_tree_branch(ui, &open, 0, &part);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    valid &= !draw_tree_branch(ui, &open, 0, &part) && open == 0;
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_tree_branch(ui, &open, 1, &part);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    valid &= draw_tree_branch(ui, &open, 1, &part) && open == 1;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Tree cell lost its geometry or reused a released chevron press.\n");
    return valid;
}

/** Compares branches, a leaf, indentation and optional glyphs against raw CUI. */
static bool tree_cell_pixels(void) {
    enum { WIDTH = 400, HEIGHT = 230 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    static const cui_table_column column = { .title = "Name", .width = CUI_FILL };
    const char *labels[] = { "src", "ui", "button.c" };
    const char *ids[] = { "src-name", "ui-name", "button-name" };
    const char *icons[] = { CUI_ICON_FOLDER, NULL, CUI_ICON_FILE };
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, .016f);
        cui_table_begin_ex(ui, "tree-cells", &column, 1, NULL);
        for (int index = 0; index < 3; ++index) {
            cui_table_row(ui, false);
            cui_table_cell(ui);
            if (pass) {
                int open = 1 - index, part;
                fpas_cui_tree_cell(ui, ids[index], labels[index], icons[index], index, &open, 1, &part);
            } else {
                bool open = index == 0;
                cui_tree_cell(ui, labels[index], index, index < 2, index < 2 ? &open : NULL, icons[index]);
            }
        }
        cui_table_end(ui);
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool matched = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!matched) fprintf(stderr, "FPAS tree cell rendering differs from direct CUI.\n");
    return matched;
}

/** Compares logical Start and End with physical Left in the RTL grid adapter. */
static bool grid_alignment(void) {
    enum { WIDTH = 440, HEIGHT = 200 };
    static unsigned char pixels[2][WIDTH * HEIGHT * 4];
    const char *titles[] = { "Start", "Left", "End" };
    const float widths[] = { 130, 130, 130 };
    const int flags[] = { CUI_TEXT_START, CUI_TEXT_LEFT, CUI_TEXT_END };
    for (int pass = 0; pass < 2; ++pass) {
        cui_sw_renderer *renderer = cui_sw_create();
        const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer), .direction = CUI_DIRECTION_RTL };
        cui_ctx *ui = cui_create(&settings);
        if (!ui) { cui_sw_destroy(renderer); return false; }
        cui_set_viewport(ui, WIDTH, HEIGHT, 1);
        cui_begin_frame(ui, .016f);
        int range[3], headers[3];
        if (pass) {
            int state[] = { -2, 0, -1, -1 };
            void *grid = fpas_cui_data_grid_begin(ui, "grid", titles, widths, flags, 3, 1,
                NULL, state, 2, 160, 40, 3, 88, 1, range, headers);
            fpas_cui_data_grid_row(ui, 0, "row", NULL);
            for (int i = 0; i < 3; ++i) fpas_cui_data_grid_cell(ui, "Value");
            fpas_cui_data_grid_end(ui, grid, NULL, state);
        } else {
            const cui_grid_column columns[] = {
                { .title = titles[0], .width = widths[0], .align = CUI_TEXT_START },
                { .title = titles[1], .width = widths[1], .align = CUI_TEXT_LEFT },
                { .title = titles[2], .width = widths[2], .align = CUI_TEXT_END }
            };
            cui_data_grid_begin(ui, "grid", columns, 3, 1, &range[0], &range[1],
                .width = CUI_PCT(88), .height = CUI_PX(160), .row_height = 40);
            cui_data_grid_row(ui, 0);
            for (int i = 0; i < 3; ++i) cui_data_grid_cell_text(ui, "Value");
            cui_data_grid_end(ui);
        }
        cui_end_frame(ui);
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[pass], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
    const bool matched = memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0;
    if (!matched) fprintf(stderr, "FPAS grid column alignment differs from direct CUI.\n");
    return matched;
}

static int grid_frame(cui_ctx *ui, int count, uint8_t *selection, int *state, int *range) {
    const char *titles[] = {"Name"};
    const float widths[] = {180};
    const int columns[] = {CUI_TEXT_LEFT};
    int header, clicked = -1;
    cui_begin_frame(ui, .016f);
    void *grid = fpas_cui_data_grid_begin(ui, "grid", titles, widths, columns, 1, count,
        selection, state, 2, 160, 40, 2, 260, 1, range, &header);
    for (int row = range[0]; row < range[1]; ++row) {
        int activated;
        fpas_cui_data_grid_row(ui, row, "row", &activated);
        if (activated) clicked = row;
        fpas_cui_data_grid_cell(ui, "Example");
    }
    fpas_cui_data_grid_end(ui, grid, selection, state);
    cui_end_frame(ui);
    return clicked;
}

static void grid_click(cui_ctx *ui, float x, float y) {
    cui_input_mouse_move(ui, x, y);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
}

static bool grid_checkbox_anchor(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 400, 260, 1);
    cui_set_reduced_motion(ui, true);
    uint8_t selection[4] = {0};
    int range[3], state[] = {-2, 0, -1, -1};
    grid_frame(ui, 4, selection, state, range);
    grid_click(ui, 22, 101);
    bool valid = grid_frame(ui, 4, selection, state, range) == -1 &&
        state[3] == 1 && selection[1] && !selection[2];

    grid_frame(ui, 4, selection, state, range);
    cui_input_key(ui, CUI_KEY_NONE, false, CUI_MOD_SHIFT);
    grid_click(ui, 100, 141);
    valid = grid_frame(ui, 4, selection, state, range) == 2 &&
        !selection[0] && selection[1] && selection[2] && !selection[3] && valid;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Grid checkbox did not set the next Shift-click origin.\n");
    return valid;
}

static bool grid_reveals_bound_cursor(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 400, 260, 1);
    cui_set_reduced_motion(ui, true);
    int range[3], state[] = {-2, 0, 50, -1};
    grid_frame(ui, 80, NULL, state, range);
    bool valid = range[0] <= 50 && range[1] > 50 &&
        fabsf(fpas_cui_node_rect(ui, range[2]).w - 260) < 1;
    state[2] = -1;
    grid_frame(ui, 80, NULL, state, range);
    state[2] = 4;
    grid_frame(ui, 80, NULL, state, range);
    valid = range[0] <= 4 && range[1] > 4 && valid;
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "Grid did not reveal its first model cursor.\n");
    return valid;
}

static void grid_headers(cui_ctx *ui, int count, cui_rect *parts) {
    const char *titles[] = {"Name", "Team"};
    const float widths[] = {180, 130};
    const int flags[] = {0, 0};
    int headers[2], range[3], state[] = {-2, 0, -1, -1};
    cui_begin_frame(ui, .016f);
    void *grid = fpas_cui_data_grid_begin(ui, "grid", titles, widths, flags, count, 0,
        NULL, state, 2, 160, 40, 2, 260, 1, range, headers);
    fpas_cui_data_grid_end(ui, grid, NULL, state);
    cui_end_frame(ui);
    for (int index = 0; index < count; ++index)
        parts[index] = fpas_cui_node_rect(ui, headers[index]);
}

static bool grid_initializes_appended_columns(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 400, 260, 1);
    cui_set_reduced_motion(ui, true);
    cui_rect parts[2];
    grid_headers(ui, 1, parts);
    grid_headers(ui, 1, parts);
    cui_input_mouse_move(ui, 180, 20);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    grid_headers(ui, 1, parts);
    cui_input_mouse_move(ui, 200, 20);
    grid_headers(ui, 1, parts);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    grid_headers(ui, 1, parts);
    grid_headers(ui, 2, parts);
    cui_destroy(ui);
    const bool valid = fabsf(parts[0].w - 200) < 1 && fabsf(parts[1].w - 130) < 1;
    if (!valid) fprintf(stderr, "Grid lost resized or appended widths: existing=%g appended=%g\n",
        parts[0].w, parts[1].w);
    return valid;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool shrink = grid_shrink(adapter);
    const bool filter = filter_add(adapter);
    const bool tree = !adapter || tree_lazy_load();
    const bool alignment = !adapter || grid_alignment();
    const bool grid_controls = !adapter ||
        (grid_checkbox_anchor() && grid_reveals_bound_cursor() && grid_initializes_appended_columns());
    const bool tree_cell = !adapter || (tree_cell_releases_chevron() && tree_cell_pixels());
    const bool options = !adapter || filter_options();
    return shrink && filter && tree && alignment && grid_controls && tree_cell && options ? 0 : 1;
}
