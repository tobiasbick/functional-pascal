// [Documentation](../../../../docs/ui/gui/inputs/README.md)
/* Exercise upstream input fixes through the FPAS adapters, without a window. */
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

typedef void (*draw_fn)(cui_ctx *, void *);
static int failures;
#define CHECK(test) do { if (!(test)) { fprintf(stderr, "line %d: %s\n", __LINE__, #test); ++failures; } } while (0)

static char clipboard[128];
static const char *get_clipboard(void *user) { (void)user; return clipboard; }
static void set_clipboard(void *user, const char *text) { (void)user; snprintf(clipboard, sizeof clipboard, "%s", text); }

static void frames(cui_ctx *ui, draw_fn draw, void *state, int count) {
    for (int f = 0; f < count; ++f) {
        cui_begin_frame(ui, .04f);
        fpas_cui_host_begin(ui);
        draw(ui, state);
        cui_end_frame(ui);
    }
}

static void key(cui_ctx *ui, draw_fn draw, void *state, cui_key code, int mods) {
    cui_input_key(ui, code, true, mods);
    cui_input_key(ui, code, false, mods);
    frames(ui, draw, state, 1);
}

static void text(cui_ctx *ui, draw_fn draw, void *state, const char *value) {
    cui_input_key(ui, CUI_KEY_NONE, false, 0);
    cui_input_text(ui, value);
    frames(ui, draw, state, 1);
}

static void number(cui_ctx *ui, void *state) {
    int nodes[3];
    fpas_cui_number_field(ui, "quantity", state, 0, 99, NULL, NULL, 0, 0, 1, 0, 0, 0, nodes);
}

static void check_copied_number(cui_ctx *ui, double *value) {
    clipboard[0] = '\0';
    key(ui, number, value, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, number, value, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "99") == 0);
}

static void check_number(cui_ctx *ui) {
    double value = 3;
    frames(ui, number, &value, 2);
    key(ui, number, &value, CUI_KEY_TAB, 0);
    key(ui, number, &value, CUI_KEY_UP, 0);
    CHECK(value == 4);
    key(ui, number, &value, CUI_KEY_A, CUI_MOD_PRIMARY);
    text(ui, number, &value, "999");
    key(ui, number, &value, CUI_KEY_ENTER, 0);
    CHECK(value == 99);
    check_copied_number(ui, &value);
    text(ui, number, &value, "nan");
    CHECK(isfinite(value) && value == 99);
    key(ui, number, &value, CUI_KEY_ENTER, 0);
    check_copied_number(ui, &value);
    text(ui, number, &value, "7x");
    CHECK(value == 99);
    key(ui, number, &value, CUI_KEY_TAB, 0);
    key(ui, number, &value, CUI_KEY_TAB, CUI_MOD_SHIFT);
    check_copied_number(ui, &value);
}

typedef struct { double value; int decimals; } number_precision;

static void precise_number(cui_ctx *ui, void *state) {
    number_precision *number = state;
    int nodes[3];
    fpas_cui_number_field(ui, "number", &number->value, 0, 10, NULL, NULL, .01,
        number->decimals, 1, 0, 0, 0, nodes);
}

static void check_number_precision(cui_ctx *ui) {
    number_precision number = {1.25, 1};
    frames(ui, precise_number, &number, 2);
    key(ui, precise_number, &number, CUI_KEY_TAB, 0);
    key(ui, precise_number, &number, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, precise_number, &number, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "1.2") == 0);

    number.decimals = 3;
    frames(ui, precise_number, &number, 1);
    key(ui, precise_number, &number, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, precise_number, &number, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "1.250") == 0 && number.value == 1.25);

    number.decimals = 16;
    frames(ui, precise_number, &number, 1);
    key(ui, precise_number, &number, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, precise_number, &number, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "1.2500000000000000") == 0 && number.value == 1.25);
}

typedef struct { bool parent_disabled, enabled; int result; } picker_state;

static int empty_folder(void *user, const char *path, cui_file_entry *entries, int capacity,
    char *names, size_t name_capacity, bool hidden) {
    (void)user; (void)path; (void)entries; (void)capacity; (void)names; (void)name_capacity; (void)hidden;
    return 0;
}

static void picker(cui_ctx *ui, void *state) {
    picker_state *picker = state;
    char path[1024];
    int nodes[5];
    cui_push_disabled(ui, picker->parent_disabled);
    picker->result = fpas_cui_file_picker(ui, "picker", 1, picker->enabled, 0, NULL,
        NULL, NULL, NULL, NULL, empty_folder, NULL, path, sizeof path, nodes);
    cui_pop_disabled(ui);
}

static void check_disabled_picker(cui_ctx *ui) {
    picker_state state = {.enabled = true};
    frames(ui, picker, &state, 2);
    state.parent_disabled = true;
    key(ui, picker, &state, CUI_KEY_ESCAPE, 0);
    CHECK(state.result == 0);
    state.parent_disabled = false;
    state.enabled = false;
    key(ui, picker, &state, CUI_KEY_ESCAPE, 0);
    CHECK(state.result == 0);
    state.enabled = true;
    key(ui, picker, &state, CUI_KEY_ESCAPE, 0);
    CHECK(state.result == 1);
}

typedef struct {
    cui_id id;
    int result;
    float height;
    int width_kind;
    float width;
    bool compact;
    bool enabled;
} dropzone_state;

static void dropzone(cui_ctx *ui, void *state) {
    dropzone_state *s = state;
    int node;
    cui_column(ui, .width = CUI_PX(240), .height = CUI_PX(60), .clip = true) {
        s->id = cui_make_id(ui, "files");
        s->result = fpas_cui_dropzone(ui, "files", NULL, NULL, "png;pdf", NULL,
            s->height, s->width_kind, s->width, s->compact, s->enabled, 0, &node);
        if (s->result > 0) {
            const char *image = cui_dropped_file(ui, 0);
            const char *document = cui_dropped_file(ui, 1);
            CHECK(s->result == 2);
            CHECK(image && strcmp(image, "cover.PNG") == 0);
            CHECK(document && strcmp(document, "notes.pdf") == 0);
            CHECK(cui_dropped_file(ui, 2) == NULL);
        }
    }
}

static void check_dropzone(cui_ctx *ui) {
    dropzone_state state = {.enabled = true};
    frames(ui, dropzone, &state, 1);
    cui_rect rect = cui_get_rect(ui, state.id);
    CHECK(rect.w == 240 && rect.h == 168);

    state.height = 120;
    state.width_kind = 2;
    state.width = 224;
    frames(ui, dropzone, &state, 1);
    rect = cui_get_rect(ui, state.id);
    CHECK(rect.w == 224 && rect.h == 120);
    const char *files[] = {"intro.txt", "cover.PNG", "notes.pdf"};
    const float x = rect.x + rect.w / 2;
    cui_input_file_drop(ui, files, 3, x, rect.y + 90);
    frames(ui, dropzone, &state, 1);
    CHECK(state.result == 0);
    cui_input_file_drop(ui, files, 3, x, rect.y + 30);
    frames(ui, dropzone, &state, 1);
    CHECK(state.result == 2);
    frames(ui, dropzone, &state, 1);
    CHECK(state.result == 0);

    cui_input_file_drop(ui, files, 1, x, rect.y + 30);
    frames(ui, dropzone, &state, 1);
    CHECK(state.result == 0);
    state.enabled = false;
    cui_input_file_drop(ui, files, 3, x, rect.y + 30);
    frames(ui, dropzone, &state, 1);
    CHECK(state.result == 0);

    state.enabled = true;
    state.compact = true;
    state.width_kind = 3;
    state.width = 50;
    frames(ui, dropzone, &state, 1);
    rect = cui_get_rect(ui, state.id);
    CHECK(rect.w == 120 && rect.h == 64);
    key(ui, dropzone, &state, CUI_KEY_TAB, 0);
    key(ui, dropzone, &state, CUI_KEY_ENTER, 0);
    CHECK(state.result == -1);
}

static void tags(cui_ctx *ui, void *state) {
    int nodes[2];
    fpas_cui_tag_input(ui, "tags", state, 128, NULL, NULL, 1, 1, 0, 1, 0, 0, nodes);
}

static void check_tags(cui_ctx *ui) {
    char value[128] = "";
    frames(ui, tags, value, 2);
    key(ui, tags, value, CUI_KEY_TAB, 0);
    text(ui, tags, value, "one,two,");
    CHECK(strcmp(value, "one") == 0);
}

typedef struct { char value[7]; int completions; } code_state;
static void code(cui_ctx *ui, void *state) {
    code_state *s = state;
    int complete;
    fpas_cui_one_time_code(ui, "code", s->value, 6, 1, 0, &complete);
    s->completions += complete;
}

static void check_code(cui_ctx *ui) {
    code_state state = {0};
    frames(ui, code, &state, 2);
    key(ui, code, &state, CUI_KEY_TAB, 0);
    text(ui, code, &state, "ABC123");
    CHECK(strcmp(state.value, "ABC123") == 0 && state.completions == 1);
    snprintf(clipboard, sizeof clipboard, "UVWXYZ");
    key(ui, code, &state, CUI_KEY_V, CUI_MOD_PRIMARY);
    CHECK(strcmp(state.value, "UVWXYZ") == 0 && state.completions == 2);
    key(ui, code, &state, CUI_KEY_V, CUI_MOD_PRIMARY);
    CHECK(state.completions == 2);
}

typedef struct { int choice, open, trigger; } select_state;
static void select_box(cui_ctx *ui, void *state) {
    select_state *s = state;
    const char *items[] = { "Alpha", "Beta" }, *icons[] = { "", "" };
    int nodes[3];
    s->open = fpas_cui_select(ui, "choice", items, icons, 2, &s->choice, NULL, NULL, 1, 1, 0, 0, 0, nodes);
    s->trigger = nodes[0];
}

static void check_select(cui_ctx *ui) {
    select_state state = {0};
    frames(ui, select_box, &state, 2);
    key(ui, select_box, &state, CUI_KEY_TAB, 0);
    key(ui, select_box, &state, CUI_KEY_ENTER, 0);
    frames(ui, select_box, &state, 2);
    CHECK(state.open && cui_wants_keyboard(ui));
    key(ui, select_box, &state, CUI_KEY_ESCAPE, 0);
    frames(ui, select_box, &state, 2);
    CHECK(!state.open && fpas_cui_focused(ui, state.trigger));
    key(ui, select_box, &state, CUI_KEY_ENTER, 0);
    frames(ui, select_box, &state, 2);
    CHECK(state.open && cui_wants_keyboard(ui) && state.choice == 0);
}

/** Compares every width kind with direct CUI both with and without a caption. */
static void check_field_widths(cui_ctx *ui) {
    const cui_size widths[] = { CUI_FIT, CUI_FILL, CUI_PX(184.5f), CUI_PCT(62.5f) };
    const char *items[] = { "Alpha", "Beta" }, *icons[] = { "", "" };
    for (int titled = 0; titled < 2; ++titled) {
        const char *label = titled ? "Caption" : NULL;
        for (int kind = 0; kind < 4; ++kind) {
            int selected = -1, parts[3];
            double value = .25;
            cui_begin_frame(ui, .04f);
            fpas_cui_host_begin(ui);
            cui_select(ui, "direct-select", items, 2, &selected, .label = label, .width = widths[kind]);
            fpas_cui_select(ui, "fpas-select", items, icons, 2, &selected, label, NULL,
                0, 1, 0, kind, widths[kind].value, parts);
            cui_number(ui, "direct-number", &value, 0, 1, .label = label, .width = widths[kind]);
            fpas_cui_number_field(ui, "fpas-number", &value, 0, 1, label, NULL,
                0, 0, 1, 0, kind, widths[kind].value, parts);
            cui_end_frame(ui);
            const float select = cui_get_rect(ui, cui_make_id(ui, "direct-select")).w;
            const float number = cui_get_rect(ui, cui_make_id(ui, "direct-number")).w;
            CHECK(select > 0 && select == cui_get_rect(ui, cui_make_id(ui, "fpas-select")).w);
            CHECK(number > 0 && number == cui_get_rect(ui, cui_make_id(ui, "fpas-number")).w);
        }
    }
}

/** Compares scalar slider formatting and width and the default range slider with CUI. */
static void check_sliders(cui_ctx *unused) {
    (void)unused;
    enum { width = 280, height = 128, stride = width * 4 };
    unsigned char pixels[2][height * stride];
    const char *format = "Level %+6.2e%%";
    for (int adapter = 0; adapter < 2; ++adapter) {
        cui_sw_renderer *sw = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
        cui_set_viewport(ui, width, height, 1);
        cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, .04f);
        fpas_cui_host_begin(ui);
        float value = 37, lower = 20, upper = 75;
        if (adapter) {
            int track;
            fpas_cui_slider(ui, "###formatted", "Level", &value, 0, 100, 0, 1, format,
                CUI_SIZE_PERCENT, 75, 1, 0, &track);
            fpas_cui_range_slider(ui, "###range", "Price", &lower, &upper, 0, 100, 1, 0, &track);
        } else {
            cui_slider(ui, "Level", &value, 0, 100, .show_value = true,
                .format = format, .width = CUI_PCT(75));
            cui_range_slider(ui, "Price", &lower, &upper, 0, 100);
        }
        cui_end_frame(ui);
        cui_sw_render(sw, cui_get_draw_data(ui), pixels[adapter], width, height, stride, CUI_SW_RGBA);
        cui_destroy(ui);
        cui_sw_destroy(sw);
    }
    CHECK(memcmp(pixels[0], pixels[1], sizeof pixels[0]) == 0);
}

typedef struct { int selection[3], nodes[4], open; bool other; } multi_selection;

static void multi_select(cui_ctx *ui, void *data) {
    multi_selection *state = data;
    const char *labels[] = { "Rust language", "Go language", "Swift" };
    const char *icons[] = { CUI_ICON_CODE, "", "" };
    state->other = cui_button(ui, "Elsewhere");
    fpas_cui_multi_select(ui, "choices", labels, icons, 3, state->selection,
        "Languages", "", 1, 1, 2, 230, 1, 0, state->nodes, &state->open);
}

static void check_multi_select_keyboard(cui_ctx *ui) {
    multi_selection state = {0};
    fpas_cui_request_focus(ui, "choices");
    frames(ui, multi_select, &state, 1);
    CHECK(fpas_cui_node_rect(ui, state.nodes[0]).w == 230);
    key(ui, multi_select, &state, CUI_KEY_ENTER, 0);
    CHECK(state.open);
    frames(ui, multi_select, &state, 1);
    key(ui, multi_select, &state, CUI_KEY_DOWN, 0);
    cui_input_key(ui, CUI_KEY_SPACE, true, 0);
    cui_input_key(ui, CUI_KEY_SPACE, false, 0);
    cui_input_text(ui, " ");
    frames(ui, multi_select, &state, 1);
    CHECK(!state.selection[0] && !state.selection[1] && !state.selection[2]);
    CHECK(state.nodes[1] >= 0 && state.nodes[2] >= 0 && state.nodes[3] < 0);
    key(ui, multi_select, &state, CUI_KEY_SPACE, 0);
    CHECK(state.open && state.selection[0]);
    key(ui, multi_select, &state, CUI_KEY_ENTER, 0);
    CHECK(state.open && !state.selection[0]);

    cui_set_reduced_motion(ui, false);
    frames(ui, multi_select, &state, 6);
    key(ui, multi_select, &state, CUI_KEY_ESCAPE, 0);
    CHECK(!state.open && fpas_cui_focused(ui, state.nodes[0]));
    fpas_cui_request_focus(ui, "Elsewhere");
    key(ui, multi_select, &state, CUI_KEY_ENTER, 0);
    CHECK(state.other && !state.open && !state.selection[0]);
}

static void area(cui_ctx *ui, void *state) {
    int nodes[3];
    cui_scroll(ui, .id = "page", .height = CUI_PX(180), .width = CUI_PX(320), .horizontal = true) {
        cui_box(ui, .height = CUI_PX(80)) {}
        fpas_cui_text_area(ui, "notes", state, 128, NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, 0, 80, 0, 0, 1, 0, 0, 0, 0, nodes);
        cui_box(ui, .width = CUI_PX(520), .height = CUI_PX(400)) {}
    }
}

static void check_area(cui_ctx *ui) {
    char value[128] = "One\nTwo\nThree\nFour\nFive\nSix\nSeven\nEight\nNine\nTen";
    frames(ui, area, value, 2);
    cui_scroll_to(ui, "page", 40);
    frames(ui, area, value, 2);
    const cui_id id = cui_make_id(ui, "notes");
    cui_rect before = cui_get_rect(ui, id);
    cui_input_mouse_move(ui, before.x + before.w / 2, before.y + before.h / 2);
    cui_input_scroll(ui, -1, -1);
    frames(ui, area, value, 1);
    cui_rect mixed = cui_get_rect(ui, id);
    CHECK(mixed.x < before.x && fabsf(mixed.y - before.y) < .01f);
    cui_input_scroll(ui, 0, 1);
    frames(ui, area, value, 1);
    CHECK(fabsf(cui_get_rect(ui, id).y - mixed.y) < .01f);
    cui_input_scroll(ui, 0, -100);
    frames(ui, area, value, 1);
    cui_input_scroll(ui, 0, -1);
    frames(ui, area, value, 1);
    CHECK(cui_get_rect(ui, id).y < mixed.y);
}

int main(void) {
    void (*checks[])(cui_ctx *) = { check_area, check_tags, check_code, check_select, check_number,
        check_number_precision, check_disabled_picker, check_dropzone, check_field_widths,
        check_multi_select_keyboard, check_sliders };
    for (size_t i = 0; i < sizeof checks / sizeof *checks; ++i) {
        cui_sw_renderer *sw = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
        const cui_platform_iface platform = { .get_clipboard = get_clipboard, .set_clipboard = set_clipboard };
        cui_set_platform(ui, &platform);
        cui_set_viewport(ui, 480, 320, 1);
        cui_set_reduced_motion(ui, true);
        checks[i](ui);
        cui_destroy(ui);
        cui_sw_destroy(sw);
    }
    printf("Input regression checks: %s (%d failures)\n", failures ? "FAILED" : "passed", failures);
    return failures ? 1 : 0;
}
