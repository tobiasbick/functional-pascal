// [Documentation](../../../../docs/ui/gui/canvas-and-bars/format-bar.md)
#include "../fpas_cui.h"
#include <stdio.h>

static int failures;

static void check(bool value, const char *message) {
    if (value) return;
    fprintf(stderr, "%s\n", message);
    failures++;
}

static cui_ctx *format_context(int width) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return NULL;
    cui_set_viewport(ui, width, 440, 1);
    cui_set_reduced_motion(ui, true);
    cui_input_mouse_leave(ui);
    return ui;
}

typedef struct {
    cui_text_format value;
    cui_format_bar_opts options;
    int flags;
} format_case;

static int format_step(cui_ctx *ui, format_case *test, cui_key key) {
    if (key) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    int result = 0;
    cui_begin_frame(ui, .016f);
    cui_column(ui, .width = CUI_FILL, .height = CUI_FILL) {
        const cui_format_bar_opts *o = &test->options;
        result = fpas_cui_format_bar(ui, "format", &test->value, o->styles, o->style_count,
            o->colors, o->color_count, o->footer, o->footer_keys, o->delete_question, test->flags);
        if (key == CUI_KEY_ENTER && !(test->flags & 1))
            check(cui_key_pressed(ui, CUI_KEY_ENTER), "Disabling the bar must leave Enter available.");
    }
    cui_end_frame(ui);
    return result;
}

static void format_options(void) {
    cui_ctx *ui = format_context(850);
    if (!ui) { check(false, "Cannot create format bar context."); return; }
    static const cui_text_style styles[] = {
        { "Header", 44, 17, CUI_FONT_MEDIUM },
        { "Big ## heading with a literal name", 27, 20, CUI_FONT_MONO }
    };
    static const cui_tone colors[] = { CUI_TONE_INFO, CUI_TONE_WARNING };
    format_case test = {
        .value = { .color = CUI_TONE_ROSE, .size = 44 },
        .options = { .styles = styles, .style_count = 2, .colors = colors, .color_count = 2,
            .footer = "Custom options", .footer_keys = "Alt P", .delete_question = "Remove selection?" },
        .flags = 1
    };
    check(format_step(ui, &test, 0) == 0, "The format bar starts without a proposal.");
    format_step(ui, &test, CUI_KEY_TAB);
    const cui_id color_trigger = cui_focused_id(ui);
    const cui_key color_openers[] = { CUI_KEY_ENTER, CUI_KEY_SPACE };
    for (int i = 0; i < 2; ++i) {
        check(format_step(ui, &test, color_openers[i]) == 0 &&
            cui_focused_id(ui) != color_trigger, "An activation key must open the grid and focus a swatch.");
        check(format_step(ui, &test, CUI_KEY_ESCAPE) == 0 && cui_focused_id(ui) == color_trigger,
            "Escape must close the grid and return focus to the trigger.");
        format_step(ui, &test, 0);
    }
    check(format_step(ui, &test, CUI_KEY_DOWN) == 0, "Opening the color menu must preserve the format.");
    format_step(ui, &test, CUI_KEY_RIGHT);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED &&
        test.value.color == CUI_TONE_WARNING, "A custom grid must start keyboard focus at its first color.");
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_SPACE) == CUI_FORMAT_CHANGED && test.value.bold,
        "Space must toggle Bold.");
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED && test.value.underline,
        "Enter must toggle Underline.");
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_LINK && !test.value.link,
        "Link must report only the action and leave the format alone.");
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_SPACE) == 0, "Space must open styles without selecting one.");
    cui_push_id(ui, "format");
    const cui_id style_popup = cui_make_id(ui, "#style") ^ 0x57713u;
    cui_pop_id(ui);
    check(cui_get_rect(ui, style_popup).w > 250,
        "The style menu must measure the full literal custom style name.");
    format_step(ui, &test, CUI_KEY_DOWN);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED &&
        test.value.style == 1 && test.value.size == 27, "The style choice must apply its index and document size.");
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    format_step(ui, &test, CUI_KEY_ENTER);
    format_step(ui, &test, CUI_KEY_RIGHT);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED &&
        test.value.align == CUI_FORMAT_ALIGN_CENTER, "Alignment must report the new format.");
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    format_step(ui, &test, CUI_KEY_ENTER);
    format_step(ui, &test, CUI_KEY_DOWN);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED &&
        test.value.list == CUI_FORMAT_LIST_NUMBERS, "List must report the chosen list kind.");
    format_step(ui, &test, CUI_KEY_ENTER);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_CHANGED &&
        test.value.list == CUI_FORMAT_LIST_NONE, "Choosing the current list kind must turn it off.");
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_ENTER) == 0, "Opening the delete question must not confirm it.");
    format_step(ui, &test, CUI_KEY_ESCAPE);
    check(format_step(ui, &test, CUI_KEY_SPACE) == 0, "Escape must cancel deletion and return focus.");
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_DELETE,
        "A second activation must confirm deletion.");
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    check(format_step(ui, &test, CUI_KEY_ENTER) == CUI_FORMAT_MORE,
        "More must report the application action.");
    cui_destroy(ui);
}

static void format_disabled_and_hidden(void) {
    cui_ctx *ui = format_context(650);
    if (!ui) { check(false, "Cannot create format bar context."); return; }
    format_case test = { .value = { .size = 44 }, .flags = 1 };
    format_step(ui, &test, 0);
    for (int i = 0; i < 8; ++i) format_step(ui, &test, CUI_KEY_TAB);
    format_step(ui, &test, CUI_KEY_ENTER);
    test.flags = 0;
    check(format_step(ui, &test, CUI_KEY_ENTER) == 0, "A disabled confirmation must not delete.");
    test.flags = 1;
    check(format_step(ui, &test, CUI_KEY_ENTER) == 0, "Re-enabling must not revive a delete question.");
    cui_destroy(ui);

    ui = format_context(650);
    if (!ui) { check(false, "Cannot create format bar context."); return; }
    test.flags = 7;
    format_step(ui, &test, 0);
    format_step(ui, &test, CUI_KEY_TAB);
    cui_push_id(ui, "format");
    const cui_id color = cui_make_id(ui, "#color");
    cui_pop_id(ui);
    check(cui_focused_id(ui) == color, "Color is the first format control.");
    for (int i = 0; i < 7; ++i) format_step(ui, &test, CUI_KEY_TAB);
    check(cui_focused_id(ui) == color, "Hidden delete and more controls must leave the Tab order.");
    cui_destroy(ui);
}

int main(void) {
    format_options();
    format_disabled_and_hidden();
    return failures != 0;
}
