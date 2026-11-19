// [Documentation](../../../../docs/ui/gui/data-display/README.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static bool same_region(const unsigned char *a, const unsigned char *b,
    int width, int height, cui_rect rect) {
    int left = (int)rect.x - 3, top = (int)rect.y - 3;
    int right = (int)(rect.x + rect.w) + 3, bottom = (int)(rect.y + rect.h) + 3;
    if (left < 0) left = 0;
    if (top < 0) top = 0;
    if (right > width) right = width;
    if (bottom > height) bottom = height;
    if (right <= left || bottom <= top) return false;
    for (int y = top; y < bottom; ++y) {
        const size_t offset = ((size_t)y * width + left) * 4;
        if (memcmp(a + offset, b + offset, (size_t)(right - left) * 4)) return false;
    }
    return true;
}

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
    cui_rect header_ring = {0}, row_ring = {0};
    for (int i = 0; i < 3; ++i) {
        if (i) {
            cui_input_key(ui, CUI_KEY_TAB, true, 0);
            cui_input_key(ui, CUI_KEY_TAB, false, 0);
        }
        cui_begin_frame(ui, .016f);
        table(ui, bridge, &header, &row);
        cui_end_frame(ui);
        if (bridge && !i) {
            header_ring = fpas_cui_node_rect(ui, header);
            row_ring = fpas_cui_node_rect(ui, row);
            /* Sample the outer halves so neighboring rings cannot share pixels. */
            focused &= header_ring.w > 0 && header_ring.h > 12 && row_ring.w > 0 && row_ring.h > 12;
            header_ring.h = header_ring.h / 2 - 3;
            row_ring.y += row_ring.h / 2 + 3;
            row_ring.h = row_ring.h / 2 - 3;
        }
        if (bridge && i) focused &= fpas_cui_focused(ui, i == 1 ? header : row) != 0;
        cui_sw_render(renderer, cui_get_draw_data(ui), pixels[i], WIDTH, HEIGHT, WIDTH * 4, CUI_SW_RGBA);
    }
    bool header_draws = memcmp(pixels[0], pixels[1], sizeof pixels[0]) != 0;
    bool row_draws = memcmp(pixels[0], pixels[2], sizeof pixels[0]) != 0;
    if (bridge) {
        header_draws = !same_region(pixels[0], pixels[1], WIDTH, HEIGHT, header_ring)
            && same_region(pixels[0], pixels[2], WIDTH, HEIGHT, header_ring);
        row_draws = !same_region(pixels[0], pixels[2], WIDTH, HEIGHT, row_ring)
            && same_region(pixels[0], pixels[1], WIDTH, HEIGHT, row_ring);
    }
    printf("%s table focus visible on header=%d row=%d\n", bridge ? "FPAS" : "CUI", header_draws, row_draws);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
    return focused && header_draws && row_draws;
}

static bool check_table_row_identity(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 400, 220, 1);
    cui_set_reduced_motion(ui, true);
    const char *titles[] = { "Title" };
    const int kinds[] = { 1 }, aligns[] = { 0 }, sortable[] = { 0 };
    const float widths[] = { 1 };
    bool valid = true;
    cui_rect target = {0};
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2) {
            cui_input_key(ui, CUI_KEY_ENTER, true, 0);
            cui_input_key(ui, CUI_KEY_ENTER, false, 0);
        }
        if (frame == 3 || frame == 6) {
            cui_input_mouse_move(ui, target.x + target.w / 2, target.y + target.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        /* Release once in the reordering frame and once after the new layout has presented. */
        if (frame == 4 || frame == 8) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);

        cui_begin_frame(ui, .016f);
        int sort = -2, descending = 0, target_node = -1;
        const int table = fpas_cui_table_begin(ui, "table", titles, kinds, widths, aligns, sortable, 1,
            0, 0, 0, 0, 1, 1, &sort, &descending, -1, NULL);
        for (int index = 0; index < 2; ++index) {
            const bool reordered = (frame >= 1 && frame < 4) || frame >= 7;
            const bool original = index == (reordered ? 1 : 0);
            const char *id = original ? "original" : "other";
            int node;
            const bool clicked = fpas_cui_table_row(ui, id, 0, 1, frame == 0 && original, &node);
            fpas_cui_table_cell(ui, id);
            fpas_cui_table_row_end(ui);
            if (original) target_node = node;
            valid &= clicked == (original && (frame == 2 || frame == 4));
            valid &= (fpas_cui_focused(ui, node) != 0) == original;
            if (frame == 4 || frame == 8)
                valid &= (fpas_cui_hovered(ui, node) != 0) == (original == (frame == 4));
        }
        fpas_cui_table_end(ui, table);
        cui_end_frame(ui);
        target = fpas_cui_node_rect(ui, target_node);
    }
    cui_destroy(ui);
    if (!valid) fprintf(stderr, "A table row lost stable focus or activation after reordering.\n");
    return valid;
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
            int node = fpas_cui_virtual_list_begin(ui, "list", count, 36, 2, 140, 1, -1, &first, &last);
            fpas_cui_virtual_list_end(ui, node);
        } else {
            cui_virtual_list_begin(ui, "list", count, 36, CUI_PX(140), &first, &last);
            cui_virtual_list_end(ui);
        }
        cui_end_frame(ui);
        if (frame == 2 && first <= 0) {
            fprintf(stderr, "%s virtual list did not scroll before shrinking.\n", bridge ? "FPAS" : "CUI");
            cui_destroy(ui);
            return false;
        }
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
    bool shown = false, enter = false;
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
            if (frame == 10) shown = panel >= 0 && query >= 0;
        } else {
            selected = cui_command_palette(ui, &open, &command, 1);
            if (frame == 10) shown = cui_wants_keyboard(ui);
        }
        enter = cui_key_pressed(ui, CUI_KEY_ENTER);
        cui_end_frame(ui);
    }
    printf("%s closing palette shown=%d selection=%d Enter available=%d\n",
        bridge ? "FPAS" : "CUI", shown, selected, enter);
    cui_destroy(ui);
    return shown && selected == -1 && enter;
}

int main(int argc, char **argv) {
    const bool bridge = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool focus = check_focus(bridge);
    const bool identity = !bridge || check_table_row_identity();
    const bool range = check_range(bridge);
    const bool list_item = !bridge || check_list_item();
    const bool palette_close = check_palette_close(bridge);
    if (!list_item) fprintf(stderr, "List item did not activate from keyboard focus.\n");
    return focus && identity && range && list_item && palette_close ? 0 : 1;
}
