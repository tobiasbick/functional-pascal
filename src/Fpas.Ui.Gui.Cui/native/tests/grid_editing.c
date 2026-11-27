// [Documentation](../../../../docs/ui/gui/data-grid/data-grid.md)
/* Exercises grid drafts through real pointer, text and keyboard input. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

typedef struct {
    cui_ctx *ui;
    int state[4], capacity, enabled, first, last, commits, edited_column;
    const char *identity;
    bool readonly, two_columns, name_visible;
    char proposal[256];
    cui_id grid;
    cui_rect row;
} fixture;

static int failures;

/** Records an observable regression without stopping the remaining cases. */
static void check(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "%s\n", message); ++failures; }
}

/** Builds fresh model buffers each frame, deliberately rejecting all proposals. */
static void draw(fixture *test) {
    const char *titles[] = { "Name", "Role" };
    const float widths[] = { 240, 240 };
    const int flags[] = { test->two_columns ? 1 << 5 : 0, 0 };
    const int column_count = test->two_columns ? 2 : 1;
    int range[3], headers[2], first_node = -1;
    cui_begin_frame(test->ui, .016f);
    void *grid = fpas_cui_data_grid_begin(test->ui, "grid", titles, widths, flags, column_count,
        100, NULL, test->state, 2, 160, 32, 1, 0, test->enabled, range, headers);
    test->first = range[0]; test->last = range[1];
    test->grid = fpas_cui_node_key(test->ui, range[2]);
    test->name_visible = headers[0] >= 0;
    for (int row = range[0]; row < range[1]; ++row) {
        char id[32];
        snprintf(id, sizeof id, "row-%d", row);
        const int node = fpas_cui_data_grid_row(test->ui, row, row ? id : test->identity, NULL);
        if (!row) first_node = node;
        for (int column = 0; column < column_count; ++column) {
            const char *caption = column == 0 ? "Alpha" : "Beta";
            char value[256] = {0};
            strcpy(value, caption);
            fpas_cui_data_grid_cell(test->ui, test->readonly ? value : NULL);
            if (!test->readonly && column == 0) cui_avatar(test->ui, caption, .size = 24);
            if (!test->readonly && fpas_cui_data_grid_edit(test->ui, value, test->capacity)) {
                ++test->commits;
                test->edited_column = column;
                memcpy(test->proposal, value, sizeof value);
            } else {
                check(!strcmp(value, caption), "An uncommitted draft must leave model text unchanged.");
            }
        }
    }
    const int activated = fpas_cui_data_grid_end(test->ui, grid, NULL, test->state);
    if (!test->readonly)
        check(activated == -1, "A double-click in an editable cell must only open its editor.");
    cui_end_frame(test->ui);
    if (first_node >= 0) test->row = fpas_cui_node_rect(test->ui, first_node);
}

/** Creates a settled virtual grid with one editable column. */
static fixture create(bool two_columns) {
    fixture test = { .ui = cui_create(NULL), .state = { -2, 0, -1, -1 },
        .capacity = 64, .enabled = 1, .identity = "row-0", .two_columns = two_columns };
    cui_set_viewport(test.ui, two_columns ? 600 : 400, 260, 1);
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
    fixture test = create(false);
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

/** Commits a complete UTF-8 prefix on pointer blur without splitting a character. */
static void commit_on_blur(void) {
    fixture test = create(false);
    test.capacity = 6;
    draft(&test, "T\xc3\xa9!\xe2\x82\xac");
    cui_input_mouse_move(test.ui, 380, 230);
    cui_input_mouse_button(test.ui, CUI_MOUSE_LEFT, true); draw(&test);
    cui_input_mouse_button(test.ui, CUI_MOUSE_LEFT, false); draw(&test);
    check(test.commits == 1 && !strcmp(test.proposal, "T\xc3\xa9!"),
        "Clicking away must commit the longest complete UTF-8 prefix exactly once.");
    cui_destroy(test.ui);
}

/** Discards undo history on Escape or retirement of the grid. */
static void cancel_and_reopen(bool retire) {
    fixture test = create(false);
    draft(&test, "First draft");
    key(&test, CUI_KEY_HOME, 0);
    cui_input_text(test.ui, "Second draft "); draw(&test);
    if (retire) {
        fpas_cui_forget(test.ui, test.grid);
        draw(&test);
        key(&test, CUI_KEY_TAB, 0);
    } else {
        key(&test, CUI_KEY_ESCAPE, 0);
    }
    check(test.commits == 0, "Closing an unfinished edit must discard the draft.");
    key(&test, CUI_KEY_ENTER, 0);
    key(&test, CUI_KEY_Z, CUI_MOD_PRIMARY);
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 1 && !strcmp(test.proposal, "Alpha"),
        "Undo after reopening must not recover text from a canceled draft.");
    cui_destroy(test.ui);
}

/** Recovers keyboard editing in the first visible column after a resize hides an active draft. */
static void skip_hidden_editor(void) {
    fixture test = create(true);
    check(test.name_visible, "The first editor must initially be visible.");
    key(&test, CUI_KEY_TAB, 0);
    key(&test, CUI_KEY_ENTER, 0);
    cui_input_text(test.ui, "Discarded draft"); draw(&test);
    cui_set_viewport(test.ui, 320, 260, 1);
    draw(&test); draw(&test);
    check(!test.name_visible, "The resize must hide the editable name column.");
    key(&test, CUI_KEY_ENTER, 0);
    cui_input_text(test.ui, "Visible role"); draw(&test);
    key(&test, CUI_KEY_ENTER, 0);
    check(test.commits == 1 && test.edited_column == 1 && !strcmp(test.proposal, "Visible role"),
        "Enter must skip a hidden editor and confirm only the visible draft.");
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
    fixture test = create(false);
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
    commit_on_blur();
    cancel_and_reopen(false);
    cancel_and_reopen(true);
    skip_hidden_editor();
    for (int reason = 0; reason < 5; ++reason) cancel(reason);
    return failures ? 1 : 0;
}
