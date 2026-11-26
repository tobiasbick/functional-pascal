// [Documentation](../../../../docs/ui/gui/data-grid/README.md)
/* Direct CUI reproductions and focused checks of the adapters. */
#include "../fpas_cui.h"
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
                NULL, state, 2, 160, 40, 1, range, &header);
            for (int row = range[0]; row < range[1]; ++row) {
                fpas_cui_data_grid_row(ui, row, "row");
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
    const char *names[] = { "Team" }, *icons[] = { "" }, *values[] = { "Engine" };
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
                NULL, state, 2, 160, 40, 1, range, headers);
            fpas_cui_data_grid_row(ui, 0, "row");
            for (int i = 0; i < 3; ++i) fpas_cui_data_grid_cell(ui, "Value");
            fpas_cui_data_grid_end(ui, grid, NULL, state);
        } else {
            const cui_grid_column columns[] = {
                { .title = titles[0], .width = widths[0], .align = CUI_TEXT_START },
                { .title = titles[1], .width = widths[1], .align = CUI_TEXT_LEFT },
                { .title = titles[2], .width = widths[2], .align = CUI_TEXT_END }
            };
            cui_data_grid_begin(ui, "grid", columns, 3, 1, &range[0], &range[1],
                .height = CUI_PX(160), .row_height = 40);
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

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool shrink = grid_shrink(adapter);
    const bool filter = filter_add(adapter);
    const bool tree = !adapter || tree_lazy_load();
    const bool alignment = !adapter || grid_alignment();
    return shrink && filter && tree && alignment ? 0 : 1;
}
