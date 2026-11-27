// [Documentation](../../../../docs/ui/gui/voice/composer.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static int failures;

static void check(bool value, const char *message) {
    if (value) return;
    fprintf(stderr, "%s\n", message);
    ++failures;
}

typedef struct {
    cui_ctx *ui;
    char draft[128];
    int selected, flags, width_kind, nodes[6];
    float width;
    bool covered, enter_available;
} composer_case;

static composer_case composer(bool reduced) {
    composer_case test = { .ui = cui_create(NULL), .flags = 5, .width_kind = 1 };
    cui_set_viewport(test.ui, 640, 480, 1);
    cui_set_reduced_motion(test.ui, reduced);
    return test;
}

static void press(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static int frame(composer_case *test, int focus, cui_key key) {
    static const cui_model models[] = {
        { "First", "First model", NULL, NULL }, { "Second", "Second model", NULL, NULL }
    };
    cui_ctx *ui = test->ui;
    if (key) press(ui, key);
    if (test->covered) cui_popover_open(ui, "cover");
    cui_begin_frame(ui, .016f);
    int removed = -1;
    const int result = fpas_cui_composer(ui, "composer", test->draft, sizeof test->draft,
        "Message", NULL, 0, models, 2, &test->selected,
        NULL, 0, 0, 0, test->width_kind, test->width, test->flags, focus, &removed, test->nodes);
    test->enter_available = cui_key_pressed(ui, CUI_KEY_ENTER);
    if (test->covered && cui_popover_begin(ui, "cover", NULL)) {
        cui_label(ui, "Overlay");
        cui_popover_end(ui);
    }
    cui_end_frame(ui);
    return result;
}

static void click_send(composer_case *test) {
    const cui_rect bounds = fpas_cui_node_rect(test->ui, test->nodes[1]);
    cui_input_mouse_move(test->ui, bounds.x + bounds.w * .5f, bounds.y + bounds.h * .5f);
    cui_input_mouse_button(test->ui, CUI_MOUSE_LEFT, true);
    cui_input_mouse_button(test->ui, CUI_MOUSE_LEFT, false);
}

static void send_after_queued_edit(void) {
    composer_case test = composer(true);
    frame(&test, 0, CUI_KEY_NONE);
    cui_input_text(test.ui, "new draft");
    click_send(&test);
    check(frame(&test, -1, CUI_KEY_NONE) == CUI_COMPOSER_SEND && !strcmp(test.draft, "new draft"),
        "Clicking Send must submit text queued in the same frame.");
    frame(&test, 0, CUI_KEY_NONE);
    frame(&test, -1, CUI_KEY_END);
    for (unsigned i = 0; i < strlen("new draft"); ++i) press(test.ui, CUI_KEY_BACKSPACE);
    click_send(&test);
    check(frame(&test, -1, CUI_KEY_NONE) == 0 && test.draft[0] == 0,
        "Clicking Send must use the draft after queued deletion.");
    frame(&test, 1, CUI_KEY_NONE);
    check(frame(&test, -1, CUI_KEY_ENTER) == 0, "Enter on empty Send must leave it inactive.");
    cui_destroy(test.ui);
}

static void open_second_model(composer_case *test, cui_key opener) {
    frame(test, 5, CUI_KEY_NONE);
    frame(test, -1, opener);
    for (int i = 0; i < 20; ++i) frame(test, -1, CUI_KEY_NONE);
    frame(test, -1, CUI_KEY_DOWN);
    frame(test, -1, CUI_KEY_DOWN);
}

static void close_model_menu(void) {
    composer_case test = composer(false);
    open_second_model(&test, CUI_KEY_ENTER);
    press(test.ui, CUI_KEY_ESCAPE);
    check(frame(&test, -1, CUI_KEY_ENTER) == 0 && test.selected == 0 && test.enter_available,
        "Escape must close the model menu without committing a queued Enter.");
    cui_destroy(test.ui);

    test = composer(false);
    open_second_model(&test, CUI_KEY_ENTER);
    test.flags &= ~1;
    check(frame(&test, -1, CUI_KEY_ENTER) == 0 && test.selected == 0 && test.enter_available,
        "A disabled composer must leave the model index and Enter alone.");
    cui_destroy(test.ui);

    test = composer(false);
    open_second_model(&test, CUI_KEY_SPACE);
    check(frame(&test, -1, CUI_KEY_SPACE) == 0 && test.selected == 0,
        "Space must keep the open model menu focused without selecting a model.");
    check(frame(&test, -1, CUI_KEY_ENTER) == CUI_COMPOSER_MODEL && test.selected == 1,
        "Enter must choose the highlighted model while the menu is active.");
    cui_destroy(test.ui);
}

static void popup_owns_input(void) {
    composer_case test = composer(true);
    strcpy(test.draft, "Existing draft");
    frame(&test, 0, CUI_KEY_NONE);
    const cui_id editor = cui_focused_id(test.ui);
    test.covered = true;
    cui_input_text(test.ui, "covered");
    check(frame(&test, -1, CUI_KEY_NONE) == 0 && !strcmp(test.draft, "Existing draft"),
        "A popup must keep queued text from reaching the draft below it.");
    check(frame(&test, -1, CUI_KEY_ENTER) == 0 &&
        test.enter_available && cui_focused_id(test.ui) == editor,
        "A popup must keep Enter without stealing the composer's focus.");
    test.covered = false;
    cui_popover_close(test.ui, "cover");
    frame(&test, 1, CUI_KEY_NONE);
    check(frame(&test, -1, CUI_KEY_SPACE) == CUI_COMPOSER_SEND,
        "Space must activate the focused Send button when there is no popup.");
    test.covered = true;
    check(frame(&test, -1, CUI_KEY_SPACE) == 0,
        "Space must not activate a composer button below a popup.");
    test.covered = false;
    cui_popover_close(test.ui, "cover");
    check(frame(&test, -1, CUI_KEY_SPACE) == CUI_COMPOSER_SEND,
        "Closing the popup must leave the focused button usable.");
    cui_destroy(test.ui);
}

static void cancel_precedes_send(void) {
    composer_case test = composer(true);
    test.flags |= 8;
    frame(&test, 1, CUI_KEY_NONE);
    press(test.ui, CUI_KEY_ESCAPE);
    check(frame(&test, -1, CUI_KEY_ENTER) == CUI_COMPOSER_CANCEL,
        "Cancel and Send in the same frame must discard the take once.");
    check(frame(&test, -1, CUI_KEY_ENTER) == CUI_COMPOSER_VOICE,
        "A separate Enter can still send a model-owned take.");
    cui_destroy(test.ui);
}

static void composer_width(void) {
    composer_case test = composer(true);
    const cui_id key = cui_make_id(test.ui, "composer");
    frame(&test, -1, CUI_KEY_NONE);
    check(cui_get_rect(test.ui, key).w == 640, "Fill must use the available width.");
    test.width_kind = 0;
    frame(&test, -1, CUI_KEY_NONE);
    const float content = cui_get_rect(test.ui, key).w;
    check(content > 0 && content < 640, "Content must size the composer from its controls.");
    test.width_kind = 3;
    test.width = 50;
    frame(&test, -1, CUI_KEY_NONE);
    check(cui_get_rect(test.ui, key).w == 320, "A percentage must measure the parent.");
    test.width_kind = 2;
    test.width = 540;
    strcpy(test.draft, "A long line that wraps when a wide composer becomes narrow without any new keyboard or mouse input.");
    frame(&test, -1, CUI_KEY_NONE);
    frame(&test, -1, CUI_KEY_NONE);
    const float wide = cui_get_rect(test.ui, key).h;
    test.width = 180;
    frame(&test, -1, CUI_KEY_NONE);
    check(cui_get_rect(test.ui, key).w == 180 && cui_needs_redraw(test.ui),
        "A width change must request the frame that measures the wrapped draft.");
    frame(&test, -1, CUI_KEY_NONE);
    check(cui_get_rect(test.ui, key).h > wide && !cui_needs_redraw(test.ui),
        "The composer must grow for the wrapped draft and then stop requesting frames.");
    cui_destroy(test.ui);
}

int main(void) {
    send_after_queued_edit();
    close_model_menu();
    popup_owns_input();
    cancel_precedes_send();
    composer_width();
    return failures != 0;
}
