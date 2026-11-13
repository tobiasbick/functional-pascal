#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static void table(cui_ctx *ui, bool bridge, int *header, int *row) {
    const cui_table_column column = { "Title", CUI_FILL, CUI_TEXT_START, true };
    cui_table_sort order = { 0, false };
    if (bridge) {
        const char *titles[] = { "Title" };
        const int kind[] = { 1 }, align[] = { 0 }, sortable[] = { 1 };
        const float weight[] = { 1 };
        int sort = 0, desc = 0;
        int node = fpas_cui_table_begin(ui, "table", titles, kind, weight, align, sortable, 1,
            0, 0, 0, 0, 1, 1, &sort, &desc, -1, header);
        fpas_cui_table_row(ui, "row", 0, 1, 0, row);
        fpas_cui_table_cell(ui, "Entry");
        fpas_cui_table_row_end(ui);
        fpas_cui_table_end(ui, node);
    } else {
        const cui_table_opts options = { .sort = &order, .selectable = true };
        cui_table_begin_ex(ui, "table", &column, 1, &options);
        cui_table_row(ui, false);
        cui_table_cell_text(ui, "Entry");
        cui_table_end(ui);
    }
}

static bool check_focus(bool bridge) {
    enum { WIDTH = 400, HEIGHT = 220 };
    static unsigned char pixels[3][WIDTH * HEIGHT * 4];
    int header = -1, row = -1;
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc options = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&options);
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    bool focused = true;
    for (int i = 0; i < 3; ++i) {
        if (i) {
            cui_input_key(ui, CUI_KEY_TAB, true, 0);
            cui_input_key(ui, CUI_KEY_TAB, false, 0);
        }
        cui_begin_frame(ui, .016f);
        table(ui, bridge, &header, &row);
        cui_end_frame(ui);
        if (bridge && i) focused &= fpas_cui_focused(ui, i == 1 ? header : row) != 0;
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[i], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    }
    bool header_draws = memcmp(pixels[0], pixels[1], sizeof pixels[0]) != 0;
    bool row_draws = memcmp(pixels[0], pixels[2], sizeof pixels[0]) != 0;
    printf("%s table focus visible on header=%d row=%d\n", bridge ? "FPAS" : "CUI", header_draws, row_draws);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    return focused && header_draws && row_draws;
}

static bool check_range(bool bridge) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 400, 220, 1);
    cui_set_reduced_motion(ui, true);
    int first = 0, last = 0;
    for (int frame = 0; frame < 4; ++frame) {
        cui_begin_frame(ui, .016f);
        const int count = frame == 3 ? 1 : 100;
        if (frame == 1) cui_scroll_to(ui, "list", 2000);
        if (bridge) {
            int node = fpas_cui_virtual_list_begin(ui, "list", count, 36, 2, 140, 1, &first, &last);
            fpas_cui_virtual_list_end(ui, node);
        } else {
            cui_virtual_list_begin(ui, "list", count, 36, CUI_PX(140), &first, &last);
            cui_virtual_list_end(ui);
        }
        cui_end_frame(ui);
    }
    printf("%s virtual range after shrink [%d,%d)\n", bridge ? "FPAS" : "CUI", first, last);
    cui_destroy(ui);
    return first == 0 && last == 1;
}

static bool check_list_item(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 400, 220, 1);
    cui_begin_frame(ui, .016f);
    fpas_cui_list_item(ui, "item", "Literal ## title", NULL, NULL, NULL, NULL, 0, 1, 1);
    cui_end_frame(ui);
    const bool focused = fpas_cui_focused(ui, fpas_cui_last_node(ui)) != 0;
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    cui_begin_frame(ui, .016f);
    const bool activated = fpas_cui_list_item(ui, "item", "Literal ## title",
        NULL, NULL, NULL, NULL, 0, 1, 0) != 0;
    cui_end_frame(ui);
    cui_destroy(ui);
    return focused && activated;
}

static bool check_palette_close(bool bridge) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 800, 600, 1);
    const cui_command command = { .title = "Open project" };
    const char *titles[] = { command.title }, *empty[] = { "" };
    bool open = true;
    int selected = -1;
    bool enter = false;
    for (int frame = 0; frame < 12; ++frame) {
        if (frame == 11) {
            open = false;
            cui_input_key(ui, CUI_KEY_ENTER, true, 0);
            cui_input_key(ui, CUI_KEY_ENTER, false, 0);
        }
        cui_begin_frame(ui, .016f);
        if (bridge) {
            unsigned int owner;
            int visible = open, panel, query, row;
            selected = fpas_cui_command_palette(ui, "palette", &visible, 1,
                titles, empty, empty, empty, 1, &owner, &panel, &query, &row);
        } else {
            selected = cui_command_palette(ui, &open, &command, 1);
        }
        enter = cui_key_pressed(ui, CUI_KEY_ENTER);
        cui_end_frame(ui);
    }
    printf("%s closing palette selection=%d Enter available=%d\n",
        bridge ? "FPAS" : "CUI", selected, enter);
    cui_destroy(ui);
    return selected == -1 && enter;
}

int main(int argc, char **argv) {
    const bool bridge = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool focus = check_focus(bridge);
    const bool range = check_range(bridge);
    const bool list_item = !bridge || check_list_item();
    const bool palette_close = check_palette_close(bridge);
    if (!list_item) fprintf(stderr, "List item did not activate from keyboard focus.\n");
    return focus && range && list_item && palette_close ? 0 : 1;
}
