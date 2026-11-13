#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static bool tabs(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    const cui_tab_bar_item items[] = {
        { CUI_ICON_HOUSE, "Home", 0 }, { CUI_ICON_SEARCH, "Search", 0 },
        { CUI_ICON_INBOX, "Inbox", 4 }, { CUI_ICON_USER, "Profile", 0 }
    };
    const char *icons[4], *labels[4];
    int badges[4], parts[4], selected = 0;
    for (int i = 0; i < 4; ++i) {
        icons[i] = items[i].icon; labels[i] = items[i].label; badges[i] = items[i].badge;
    }
    const cui_key input[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_RIGHT, CUI_KEY_LEFT };
    const int expected[] = { 0, 0, 1, 0 };
    bool ok = true;
    for (int i = 0; i < 4; ++i) {
        if (i) key(ui, input[i]);
        cui_begin_frame(ui, .016f);
        const bool changed = adapter
            ? fpas_cui_tab_bar(ui, "tabs", icons, labels, badges, 4, &selected, 0, 0, 2, -1, parts)
            : cui_tab_bar(ui, "tabs", items, 4, &selected);
        cui_end_frame(ui);
        if (selected != expected[i] || changed != (i >= 2)) {
            fprintf(stderr, "%s tab frame %d: selected=%d, changed=%d\n",
                adapter ? "FPAS" : "CUI", i, selected, changed);
            ok = false;
        }
        if (adapter && i && !fpas_cui_focused(ui, parts[expected[i]])) ok = false;
    }
    cui_destroy(ui);
    return ok;
}

static bool typewriter(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    const fpas_motion_text options = { .speed = 32 };
    const float elapsed[] = { .016f, 1.f, .016f, .016f, .016f };
    const int replay[] = { 0, 0, 1, 1, 1 };
    const int expected[] = { 0, 3, 0, 3, 1 };
    bool ok = true;
    for (int i = 0; i < 5; ++i) {
        if (i == 3) cui_set_reduced_motion(ui, true);
        cui_begin_frame(ui, elapsed[i]);
        const int done = fpas_cui_typewriter(ui, "type", "Stream", replay[i], &options);
        cui_end_frame(ui);
        if (done != expected[i]) {
            fprintf(stderr, "FPAS typewriter frame %d: complete=%d\n", i, done);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool number_roll(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 420, 1);
    cui_set_reduced_motion(ui, true);
    const char *samples[] = { "1234567890123456789012345", "1 milliseconds per request" };
    bool ok = true;
    for (int i = 0; i < 2; ++i) {
        cui_begin_frame(ui, .016f);
        cui_text(ui, samples[i], .tabular = true);
        cui_end_frame(ui);
        const int expected = cui_get_draw_data(ui)->vertex_count;
        cui_begin_frame(ui, .016f);
        if (adapter) {
            const fpas_motion_text options = { 0 };
            fpas_cui_number_roll(ui, "number", samples[i], &options);
        } else {
            cui_number_roll(ui, "number", samples[i]);
        }
        cui_end_frame(ui);
        const int actual = cui_get_draw_data(ui)->vertex_count;
        if (actual != expected) {
            fprintf(stderr, "%s rolling number %d: glyph geometry has %d vertices, expected %d\n",
                adapter ? "FPAS" : "CUI", i, actual, expected);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

int main(int argc, char **argv) {
    const bool adapter = argc < 2 || strcmp(argv[1], "upstream") != 0;
    const bool navigation = tabs(adapter);
    const bool replay = !adapter || typewriter();
    const bool full_number = number_roll(adapter);
    return navigation && replay && full_number ? 0 : 1;
}
