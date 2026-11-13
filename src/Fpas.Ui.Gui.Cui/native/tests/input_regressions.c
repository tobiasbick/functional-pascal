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
    fpas_cui_number_field(ui, "quantity", state, 0, 99, NULL, NULL, 1, 0, 1, 0, nodes);
}

static void check_number(cui_ctx *ui) {
    double value = 3;
    frames(ui, number, &value, 2);
    key(ui, number, &value, CUI_KEY_TAB, 0);
    key(ui, number, &value, CUI_KEY_A, CUI_MOD_PRIMARY);
    text(ui, number, &value, "999");
    key(ui, number, &value, CUI_KEY_ENTER, 0);
    CHECK(value == 99);
    key(ui, number, &value, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, number, &value, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "99") == 0);
    text(ui, number, &value, "nan");
    CHECK(isfinite(value) && value == 99);
    key(ui, number, &value, CUI_KEY_ENTER, 0);
    key(ui, number, &value, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, number, &value, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "99") == 0);
    text(ui, number, &value, "7x");
    CHECK(value == 99);
    key(ui, number, &value, CUI_KEY_TAB, 0);
    key(ui, number, &value, CUI_KEY_TAB, CUI_MOD_SHIFT);
    key(ui, number, &value, CUI_KEY_A, CUI_MOD_PRIMARY);
    key(ui, number, &value, CUI_KEY_C, CUI_MOD_PRIMARY);
    CHECK(strcmp(clipboard, "99") == 0);
}

static void tags(cui_ctx *ui, void *state) {
    int nodes[2];
    fpas_cui_tag_input(ui, "tags", state, 128, NULL, NULL, 1, 1, 0, 0, nodes);
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
    s->open = fpas_cui_select(ui, "choice", items, icons, 2, &s->choice, NULL, NULL, 1, 1, 0, nodes);
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

static void area(cui_ctx *ui, void *state) {
    int nodes[3];
    cui_scroll(ui, .id = "page", .height = CUI_PX(180), .width = CUI_PX(320), .horizontal = true) {
        cui_box(ui, .height = CUI_PX(80)) {}
        fpas_cui_text_area(ui, "notes", state, 128, NULL, NULL, NULL, NULL, NULL,
            NULL, NULL, 0, 80, 0, 0, 1, 0, nodes);
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
    void (*checks[])(cui_ctx *) = { check_area, check_tags, check_code, check_select, check_number };
    for (int i = 0; i < 5; ++i) {
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
