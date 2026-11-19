// [Documentation](../../../../docs/ui/gui/data-grid/data-grid.md)
/* Exercises grid drafts through real pointer, text and keyboard input. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

typedef struct {
    cui_ctx *ui;
    int state[4], capacity, enabled, first, last, commits;
    const char *identity;
    bool readonly;
    char proposal[256];
    cui_rect row;
} fixture;

static int failures;

/** Records an observable regression without stopping the remaining cases. */
static void check(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "%s\n", message); ++failures; }
}

/** Builds fresh model buffers each frame, deliberately rejecting all proposals. */
static void draw(fixture *test) {
    const char *titles[] = { "Name" };
    const float widths[] = { 240 };
    const int flags[] = { 0 };
    int range[3], header, first_node = -1;
    cui_begin_frame(test->ui, .016f);
    void *grid = fpas_cui_data_grid_begin(test->ui, "grid", titles, widths, flags, 1,
        100, NULL, test->state, 2, 160, 32, test->enabled, range, &header);
    test->first = range[0]; test->last = range[1];
    for (int row = range[0]; row < range[1]; ++row) {
        char id[32], value[256] = "Alpha";
        snprintf(id, sizeof id, "row-%d", row);
        const int node = fpas_cui_data_grid_row(test->ui, row, row ? id : test->identity);
        if (!row) first_node = node;
        fpas_cui_data_grid_cell(test->ui, test->readonly ? value : NULL);
        if (!test->readonly && fpas_cui_data_grid_edit(test->ui, value, test->capacity)) {
            ++test->commits;
            memcpy(test->proposal, value, sizeof value);
        } else {
            check(!strcmp(value, "Alpha"), "An uncommitted draft must leave model text unchanged.");
        }
    }
    const int activated = fpas_cui_data_grid_end(test->ui, grid, NULL, test->state);
    if (!test->readonly)
        check(activated == -1, "A double-click in an editable cell must only open its editor.");
    cui_end_frame(test->ui);
    if (first_node >= 0) test->row = fpas_cui_node_rect(test->ui, first_node);
}

/** Creates a settled virtual grid with one editable column. */
static fixture create(void) {
    fixture test = { .ui = cui_create(NULL), .state = { -2, 0, -1, -1 },
        .capacity = 64, .enabled = 1, .identity = "row-0" };
    cui_set_viewport(test.ui, 400, 260, 1);
    cui_set_reduced_motion(test.ui, true);
    draw(&test); draw(&test);
    return test;
}

/** Sends one complete key press to the following frame. */
static void key(fixture *test, cui_key code, int mods) {
    cui_input_key(test->ui, code, true, mods);
    cui_input_key(test->ui, code, false, mods);
    draw(test);
}

/** Double-clicks the first editable cell using its finalized row geometry. */
static void open_edit(fixture *test) {
    check(test->first == 0 && test->last > 0 && test->row.h > 0,
        "The first editable row must be realized before opening its editor.");
    cui_input_mouse_move(test->ui, test->row.x + 70, test->row.y + test->row.h / 2);
    for (int click = 0; click < 2; ++click) {
        cui_input_mouse_button(test->ui, CUI_MOUSE_LEFT, true); draw(test);
        cui_input_mouse_button(test->ui, CUI_MOUSE_LEFT, false); draw(test);
    }
    draw(test);
}

/** Replaces the current draft while keeping the application model unchanged. */
static void draft(fixture *test, const char *text) {
    open_edit(test);
    key(test, CUI_KEY_A, CUI_MOD_PRIMARY);
    cui_input_key(test->ui, CUI_KEY_NONE, false, 0);
    cui_input_text(test->ui, text); draw(test);
    check(test->commits == 0, "Typing must not commit a grid draft.");
}

/** Checks successful commit and rejection by leaving the application model unchanged. */
static void commit_and_reject(void) {
    fixture test = create();
    draft(&test, "Draft");
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 1 && !strcmp(test.proposal, "Draft"),
        "Enter must propose the complete edited text exactly once.");
    test.commits = 0;
    open_edit(&test);
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 1 && !strcmp(test.proposal, "Alpha"),
        "Reopening a rejected edit must use current model text, not the rejected draft.");
    cui_destroy(test.ui);
}

/** Scrolls by a real wheel event until row zero leaves or returns to realization. */
static void scroll(fixture *test, float delta) {
    cui_input_mouse_move(test->ui, 100, 100);
    cui_input_scroll(test->ui, 0, delta);
    draw(test); draw(test); draw(test);
}

/** Checks that each lifecycle transition discards an in-progress stale draft. */
static void cancel(int reason) {
    fixture test = create();
    draft(&test, "Stale");
    if (reason == 0) test.identity = "replacement-row";
    if (reason == 1) test.capacity = 32;
    if (reason == 2) test.enabled = 0;
    if (reason == 3) {
        scroll(&test, -60);
        check(test.first > 0, "Virtualization cancellation must actually unrealize the edited row.");
    }
    if (reason == 4) test.readonly = true;
    draw(&test);
    test.readonly = false;
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 0, "A canceled stale draft must not commit on Enter.");
    test.enabled = 1;
    if (reason == 3) {
        scroll(&test, 60);
        check(test.first == 0, "The edited row must return before testing its next edit.");
    }
    draw(&test);
    open_edit(&test);
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 1 && !strcmp(test.proposal, "Alpha"),
        "The next edit after cancellation must start from the current model value.");
    cui_destroy(test.ui);
}

/** Runs successful/rejected commits and each adapter-owned draft cancellation path. */
int main(void) {
    commit_and_reject();
    for (int reason = 0; reason < 5; ++reason) cancel(reason);
    return failures ? 1 : 0;
}
