// [Documentation](../../../../docs/ui/gui/more-developer-tools/README.md)
// [Terminal](../../../../docs/ui/gui/files/terminal.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int failures;

static void check(bool condition, const char *label) {
    if (!condition) { fprintf(stderr, "%s\n", label); ++failures; }
}

static cui_ctx *context(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 780, 450, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void press(cui_ctx *ui, cui_key key, int mods) {
    cui_input_key(ui, key, true, mods);
    cui_input_key(ui, key, false, mods);
}

static void markdown(void) {
    cui_ctx *ui = context();
    const char *source = "\xc3\xa9\n\n- [ ] Pending\n\n[Guide](https://example.org/guide)";
    int toggles = 0, links = 0;
    for (int frame = 0; frame < 8; ++frame) {
        if (frame == 2 || frame == 5) press(ui, CUI_KEY_TAB, 0);
        if (frame == 3 || frame == 6) press(ui, CUI_KEY_ENTER, 0);
        cui_begin_frame(ui, .016f);
        int offset, line, checked, hover_changed;
        float height;
        const char *hovered;
        const char *link = fpas_cui_markdown(ui, "notes", source, 1, 0, 0, 1, 0xff2414ecu,
            &offset, &line, &checked, &hovered, &hover_changed, &height);
        if (frame > 0) check(height > 0, "Markdown must report its previously laid-out height.");
        if (offset >= 0) {
            check(offset == 7 && line == 3 && checked, "Markdown must report the UTF-8 task offset and its proposed state.");
            ++toggles;
        }
        if (link) {
            check(strcmp(link, "https://example.org/guide") == 0, "Markdown must return the activated URL.");
            ++links;
        }
        cui_end_frame(ui);
    }
    check(toggles == 1 && links == 1, "Markdown must activate one task and one link by keyboard.");
    cui_destroy(ui);
}

static void editor(void) {
    cui_ctx *ui = context();
    char source[24] = "SELECT";
    int changes = 0;
    for (int frame = 0; frame < 9; ++frame) {
        if (frame == 2) press(ui, CUI_KEY_END, 0);
        if (frame == 3) cui_input_text(ui, " \xc3\xa9");
        if (frame == 5) memcpy(source, "FROM", 5);
        if (frame == 6) press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY);
        if (frame == 7) cui_input_key(ui, CUI_KEY_NONE, false, 0);
        if (frame == 7) cui_input_text(ui, " xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        cui_begin_frame(ui, .016f);
        fpas_code_editor_observation observation;
        int result = fpas_cui_code_editor(ui,"sql", source,sizeof source,"sql",NULL,1,0,250,0,4,
            frame == 0 ? 33 : 1, NULL, 0, &observation);
        if (frame == 0) check(observation.focused, "The editor must honor initial autofocus.");
        if (result & 1) ++changes;
        if (frame == 3) check(!strcmp(source, "SELECT \xc3\xa9") && (result & 1) &&
            observation.line == 1 && observation.lines == 1 && observation.length == 9,
            "The SQL editor must propose one complete UTF-8 insertion.");
        if (frame == 6) check(!strcmp(source, "FROM"), "Undo must not resurrect source replaced by the application.");
        if (frame == 7) check((result & 2) && !strcmp(source, "FROM"), "An oversized insertion must be refused whole.");
        cui_end_frame(ui);
    }
    check(changes == 1, "The editor must report only accepted mutations.");
    cui_destroy(ui);
}

static void diff(void) {
    cui_ctx *ui = context();
    fpas_diff_observation result = { .current = -1 };
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 2) press(ui, CUI_KEY_N, 0);
        if (frame == 4) cui_set_viewport(ui, 620, 450, 1);
        cui_begin_frame(ui, .016f);
        fpas_cui_diff_view(ui, "diff", "one\ntwo\n", "one\nthree\n", result.current, "text", NULL, NULL, 3,
            1, 0, 250, 0, 700, frame == 0 ? 17 : 1, &result);
        if (frame == 0) check(result.recomputed && result.added == 1 && result.removed == 1 && !result.approximate,
            "The diff must report line totals and a recomputation for new text.");
        if (frame == 2) check(result.current_changed, "Changing the selected hunk must report its change flag.");
        if (frame == 3) check(result.count == 1 && result.current == 0 && !result.inline_mode,
            "The split diff must report its change and respond to Next.");
        cui_end_frame(ui);
    }
    check(result.inline_mode && result.current == 0,
        "Below the configured split width, the diff must become inline and keep the selected hunk.");
    cui_destroy(ui);
}

static void hex(void) {
    cui_ctx *ui = context();
    uint8_t data[] = {0, 1, 2, 3, 4, 5};
    int start = 0, length = 0, cursor = 0;
    int edits = 0;
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 3) cui_input_text(ui, "ab");
        cui_begin_frame(ui, .016f);
        int edited_at, edited_size, hovered;
        int result = fpas_cui_hex_view(ui,"hex",data,6,&start,&length,&cursor,NULL,0,0,
            frame == 1 ? 3 : -1,16,1,0,220,0,frame == 0 ? 19 : 3,&edited_at,&edited_size,&hovered);
        if (frame == 1) check((result & 1) && start == 3 && length == 1 && cursor == 3,
            "Hex goto must select the requested byte.");
        if (result & 2) {
            check(edited_at == 3 && edited_size == 1 && data[3] == 0xab,
                "Typing two hex digits must overwrite exactly the selected byte.");
            ++edits;
        }
        cui_end_frame(ui);
    }
    check(edits == 1, "The hexadecimal edit must be reported exactly once.");
    cui_destroy(ui);
}

static void terminal(void) {
    cui_ctx *ui = context();
    int commands = 0, interrupts = 0;
    for (int frame = 0; frame < 10; ++frame) {
        if (frame == 2) cui_input_text(ui, "echo example");
        if (frame == 3 || frame == 6) press(ui, CUI_KEY_ENTER, 0);
        if (frame == 5) press(ui, CUI_KEY_UP, 0);
        if (frame == 8) press(ui, CUI_KEY_C, CUI_MOD_CTRL);
        cui_begin_frame(ui, .016f);
        int interrupt = 0;
        const char *line = fpas_cui_terminal(ui,"terminal",frame < 4 ? "Ready\n" : "Ready\nexample\n",
            0,"> ",1,0,260,0,65536,1024,frame == 0 ? 7 : 3,&interrupt);
        if (line) {
            check(!strcmp(line, "echo example"), "Submission and Up history must return the same full command.");
            ++commands;
        }
        interrupts += interrupt;
        cui_end_frame(ui);
    }
    check(commands == 2 && interrupts == 1, "Expected submission, history replay, and one interrupt.");
    cui_destroy(ui);
}

enum { TERMINAL_WIDTH = 500, TERMINAL_HEIGHT = 260, TERMINAL_BYTES = TERMINAL_WIDTH * TERMINAL_HEIGHT * 4 };

/** Compares native output, capacity changes, ANSI appends and explicit clear with the adapter. */
static void terminal_rendering(void) {
    static uint8_t expected[4][TERMINAL_BYTES], actual[TERMINAL_BYTES];
    const char *prefix = "A longer line wraps more than once at this width.\nSecond line\nProgress 0%\r\x1b[";
    char output[192];
    snprintf(output, sizeof output, "%s32m100%%\x1b[0m\n", prefix);
    for (int adapter = 0; adapter < 2; ++adapter) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)});
        cui_set_viewport(ui, TERMINAL_WIDTH, TERMINAL_HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        char bytes[192];
        cui_terminal_line lines[8];
        cui_terminal_buffer buffer;
        cui_terminal_init(&buffer, bytes, sizeof bytes, lines, 8);
        cui_terminal_write(&buffer, prefix, -1);
        for (int frame = 0; frame < 8; ++frame) {
            if (frame == 5) cui_terminal_printf(&buffer, "32m%d%%\x1b[0m\n", 100);
            if (frame == 6) {
                cui_terminal_init(&buffer, bytes, 64, lines, 2);
                cui_terminal_write(&buffer, output, -1);
            }
            if (frame == 7) cui_terminal_clear(&buffer);
            cui_begin_frame(ui, .016f);
            if (frame >= 3) {
                if (adapter) {
                    int interrupt;
                    fpas_cui_terminal(ui, "output", frame == 7 ? "" : frame >= 5 ? output : prefix, frame == 7,
                        "", 2, 230, -1, 15, frame >= 6 ? 64 : 192, frame >= 6 ? 2 : 8, 1, &interrupt);
                } else {
                    cui_terminal(ui, "output", &buffer, .width = CUI_PX(230), .height = -1, .size = 15, .no_input = true);
                }
            }
            cui_end_frame(ui);
            if (adapter && frame == 3)
                check(cui_needs_redraw(ui), "New terminal geometry must request a second wrap frame.");
            if (adapter && frame == 4)
                check(!cui_needs_redraw(ui), "Settled terminal geometry must stop requesting frames.");
            if (frame >= 4) {
                cui_sw_render(renderer, cui_get_draw_data(ui), adapter ? actual : expected[frame - 4],
                    TERMINAL_WIDTH, TERMINAL_HEIGHT, TERMINAL_WIDTH * 4, CUI_SW_RGBA);
                if (adapter) check(memcmp(actual, expected[frame - 4], TERMINAL_BYTES) == 0,
                    "Terminal wrapping, ANSI append, bounded capacities and revision clear must match CUI.");
            }
        }
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
}

/** Compares echo, interrupt and clear and verifies a disabled terminal skips Ctrl+L. */
static void terminal_shortcuts(void) {
    static uint8_t expected[3][TERMINAL_BYTES], actual[TERMINAL_BYTES];
    for (int adapter = 0; adapter < 2; ++adapter) {
        cui_sw_renderer *renderer = cui_sw_create();
        cui_ctx *ui = cui_create(&(cui_ctx_desc){.renderer = cui_sw_iface(renderer)});
        cui_set_viewport(ui, TERMINAL_WIDTH, TERMINAL_HEIGHT, 1);
        cui_set_reduced_motion(ui, true);
        char bytes[2048];
        cui_terminal_line lines[32];
        cui_terminal_buffer buffer;
        cui_terminal_init(&buffer, bytes, sizeof bytes, lines, 32);
        cui_terminal_write(&buffer, "Ready\n", -1);
        int region = 0;
        for (int frame = 0; frame < 9; ++frame) {
            if (frame == 1 || frame == 6) press(ui, CUI_KEY_TAB, 0);
            if (frame == 2) { cui_input_text(ui, "preview line"); press(ui, CUI_KEY_ENTER, 0); }
            if (frame == 3) press(ui, CUI_KEY_C, CUI_MOD_CTRL);
            if ((frame == 4 && adapter) || frame == 7) press(ui, CUI_KEY_L, CUI_MOD_CTRL);
            cui_begin_frame(ui, .016f);
            cui_push_disabled(ui, frame == 4);
            int interrupt;
            const char *command;
            if (adapter) {
                command = fpas_cui_terminal(ui, "actions", "Ready\n", 0, "", 1, 0, 210, 13,
                    (int)sizeof bytes, 32, 3, &interrupt);
            } else {
                const cui_terminal_result result = cui_terminal(ui, "actions", &buffer, .prompt = "", .height = 210, .size = 13);
                command = result.command;
                interrupt = result.interrupt;
            }
            cui_pop_disabled(ui);
            check((frame == 2 && command && !strcmp(command, "preview line")) || (frame != 2 && !command),
                "Terminal must submit the complete preview line once.");
            check(interrupt == (frame == 3), "Terminal must distinguish interrupt from clear.");
            cui_end_frame(ui);
            if (frame == 3 || frame == 6 || frame == 8) {
                cui_sw_render(renderer, cui_get_draw_data(ui), adapter ? actual : expected[region],
                    TERMINAL_WIDTH, TERMINAL_HEIGHT, TERMINAL_WIDTH * 4, CUI_SW_RGBA);
                if (adapter) check(memcmp(actual, expected[region], TERMINAL_BYTES) == 0,
                    "Terminal echo, interrupt and enabled clear must match CUI, and disabled clear must leave text intact.");
                ++region;
            }
        }
        check(memcmp(expected[1], expected[2], TERMINAL_BYTES) != 0,
            "Enabled clear must remove the echoed terminal output.");
        cui_destroy(ui);
        cui_sw_destroy(renderer);
    }
}

static void clipboard(void *user, const char *value) {
    snprintf(user, 128, "%s", value);
}

static void log_console(void) {
    cui_ctx *ui = context();
    char copied[128] = "";
    const cui_platform_iface platform = {.user = copied, .set_clipboard = clipboard};
    cui_set_platform(ui, &platform);
    const fpas_log_entry entries[] = {{"Loading\n", 0.125, CUI_LOG_INFO}, {"Ready", 1, CUI_LOG_WARN}};
    cui_push_id(ui, "log");
    cui_push_id(ui, "Clear log");
    const cui_id clear = cui_make_id(ui, CUI_ICON_TRASH_2);
    cui_pop_id(ui);
    cui_pop_id(ui);
    int clears = 0;
    for (int frame = 0; frame < 10; ++frame) {
        if (frame == 2) {
            cui_input_mouse_move(ui, 140, 65);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 3) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 4) press(ui, CUI_KEY_A, CUI_MOD_PRIMARY);
        if (frame == 5) press(ui, CUI_KEY_C, CUI_MOD_PRIMARY);
        if (frame == 6 || frame == 7) {
            const cui_rect button = cui_get_rect(ui, clear);
            check(button.w > 0, "The nonempty log must show a Clear button.");
            cui_input_mouse_move(ui, button.x + button.w / 2, button.y + button.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, frame == 6);
        }
        cui_begin_frame(ui, .016f);
        clears += fpas_cui_log_console(ui, "log", entries, clears ? 0 : 2, 1, 0, 300, 0, 3);
        cui_end_frame(ui);
    }
    check(strstr(copied, "00:00:00.125 info  Loading") && strstr(copied, "00:00:01.000 warn  Ready"),
        "Copy must return the original timestamped lines.");
    check(clears == 1, "The native Clear button must propose removing all log entries once.");
    cui_destroy(ui);
}

/* Compare updated widgets with a fresh rendering of the same application data.
   This checks visible scrollback rather than the adapter's private cache layout. */
enum { TRANSITION_WIDTH = 780, TRANSITION_HEIGHT = 450,
    TRANSITION_BYTES = TRANSITION_WIDTH * TRANSITION_HEIGHT * 4 };

static void transition_frame(cui_ctx *ui, cui_sw_renderer *renderer, const char *output,
    int revision, const fpas_log_entry *entries, int count, unsigned char *pixels) {
    cui_begin_frame(ui, .016f);
    if (output) {
        int interrupt;
        fpas_cui_terminal(ui, "transition", output, revision, "> ", 1, 0, 300, 0, 65536, 1024, 1, &interrupt);
    } else fpas_cui_log_console(ui, "transition", entries, count, 1, 0, 300, 0, 5);
    cui_end_frame(ui);
    cui_sw_render(renderer, cui_get_draw_data(ui), pixels, TRANSITION_WIDTH,
        TRANSITION_HEIGHT, TRANSITION_WIDTH * 4, CUI_SW_RGBA);
}

static void check_transition(cui_ctx *ui, cui_sw_renderer *renderer, const char *output,
    int revision, const fpas_log_entry *entries, int count, const char *message) {
    unsigned char *actual = malloc(TRANSITION_BYTES), *expected = malloc(TRANSITION_BYTES);
    check(actual && expected, "Transition pixel buffers must allocate.");
    if (actual && expected) {
        cui_sw_renderer *fresh_renderer = cui_sw_create();
        cui_ctx *fresh = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(fresh_renderer) });
        cui_set_viewport(fresh, TRANSITION_WIDTH, TRANSITION_HEIGHT, 1);
        cui_set_reduced_motion(fresh, true);
        /* Settle layout on both instances before comparing their complete output. */
        for (int frame = 0; frame < 3; ++frame) {
            transition_frame(ui, renderer, output, revision, entries, count, actual);
            transition_frame(fresh, fresh_renderer, output, revision, entries, count, expected);
        }
        check(!memcmp(actual, expected, TRANSITION_BYTES), message);
        cui_destroy(fresh);
        cui_sw_destroy(fresh_renderer);
    }
    free(actual);
    free(expected);
}

static void source_transitions(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(renderer) });
    cui_set_viewport(ui, TRANSITION_WIDTH, TRANSITION_HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    check_transition(ui, renderer, "Alpha\n", 0, NULL, 0, "Initial terminal output must match fresh scrollback.");
    check_transition(ui, renderer, "Alpha\nBeta\n", 0, NULL, 0, "Terminal append must preserve complete output.");
    check_transition(ui, renderer, "Other\nText\n", 0, NULL, 0, "Same-length terminal replacement must discard old scrollback.");
    check_transition(ui, renderer, "X\n", 0, NULL, 0, "Shrinking terminal output must discard old scrollback.");
    check_transition(ui, renderer, "X\n", 1, NULL, 0, "A terminal revision must rebuild scrollback without duplicating output.");
    fpas_cui_forget(ui, cui_make_id(ui, "transition"));
    check_transition(ui, renderer, "New\n", 1, NULL, 0, "Reinserted terminal output must start with fresh scrollback.");
    fpas_cui_forget(ui, cui_make_id(ui, "transition"));

    char text[12][96];
    fpas_log_entry entries[12];
    for (int i = 0; i < 12; ++i) {
        snprintf(text[i], sizeof text[i], "Row %02d: enough text to grow the backing text allocation beyond its initial capacity", i);
        entries[i] = (fpas_log_entry){ text[i], i, CUI_LOG_INFO };
    }
    check_transition(ui, renderer, NULL, 0, entries, 1, "Initial log entries must match a fresh console.");
    check_transition(ui, renderer, NULL, 0, entries, 12, "Log growth must preserve every row across allocation changes.");
    entries[0].text = "Replacement";
    check_transition(ui, renderer, NULL, 0, entries, 12, "A replaced log prefix must refresh the visible rows.");
    check_transition(ui, renderer, NULL, 0, entries, 2, "Shrinking a log must remove the old tail.");
    check_transition(ui, renderer, NULL, 0, entries, 3, "Appending after log replacement must preserve the new prefix.");
    fpas_cui_forget(ui, cui_make_id(ui, "transition"));
    check_transition(ui, renderer, NULL, 0, entries + 1, 2, "Reinserted log consoles must not retain previous entries.");
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}

enum { PATH_WIDTH = 640, PATH_HEIGHT = 360 };

static void render_path_menu(const char *label, unsigned char *pixels, bool disable) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    cui_set_viewport(ui, PATH_WIDTH, PATH_HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    const char *segments[] = { "project", "source" };
    cui_rect separator = {0};
    for (int frame = 0; frame < (disable ? 7 : 5); ++frame) {
        cui_begin_frame(ui, .04f);
        if (frame == 2 || frame == 3) {
            fpas_cui_move_pointer(ui, separator.x + separator.w / 2, separator.y + separator.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, frame == 2);
        }
        int parts[5], root, editing;
        uint32_t owner;
        const char *submitted;
        cui_column(ui, .width = CUI_FILL, .pad = 24) {
            cui_push_disabled(ui, disable && frame == 5);
            fpas_cui_path_bar(ui, "###location", segments, 2, NULL, NULL, NULL, 1, 1, 0,
                7, -1, &submitted, &editing, parts, &root, &owner);
            const int parent = fpas_cui_path_menu_begin(ui, "###location");
            if (parent >= 0) {
                fpas_cui_path_menu_item(ui, label, 0);
                fpas_cui_path_menu_end(ui);
            }
            if (frame == 4) check(parent == 0, "Clicking the separator must open the first segment's child menu.");
            if (frame >= 5) check(parent == -1, "Disabling the path bar must close its child menu until reopened.");
            cui_pop_disabled(ui);
        }
        cui_end_frame(ui);
        if (frame == 1) separator = fpas_cui_node_rect(ui, parts[1]);
        if (frame == 4)
            cui_sw_render(renderer, cui_get_draw_data(ui), pixels, PATH_WIDTH, PATH_HEIGHT, PATH_WIDTH * 4, CUI_SW_RGBA);
    }
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}

static void path_bar(void) {
    const size_t bytes = (size_t)PATH_WIDTH * PATH_HEIGHT * 4;
    unsigned char *plain = malloc(bytes), *literal = malloc(bytes);
    check(plain && literal, "Path menu pixel buffers must allocate.");
    if (plain && literal) {
        render_path_menu("Alpha", plain, true);
        render_path_menu("Alpha##Beta", literal, false);
        check(memcmp(plain, literal, bytes) != 0, "The child menu must render the caption suffix after ##.");
    }
    free(plain);
    free(literal);
}

static void path_editing(void) {
    const char *segments[] = { "project", "source" };
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, PATH_WIDTH, PATH_HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    for (int frame = 0; frame < 4; ++frame) {
        cui_begin_frame(ui, .016f);
        const cui_key pressed = frame == 1 || frame == 3 ? CUI_KEY_L : frame == 2 ? CUI_KEY_ENTER : CUI_KEY_NONE;
        const int mods = pressed == CUI_KEY_L ? CUI_MOD_PRIMARY : 0;
        if (pressed) {
            cui_input_key(ui, pressed, true, mods);
            cui_input_key(ui, pressed, false, mods);
        }
        int root, editing, parts[5];
        uint32_t owner;
        const char *submitted;
        const int clicked = fpas_cui_path_bar(ui, "path", segments, 2, NULL, NULL, NULL, 1, 1, 0,
            frame == 3 ? 1 : 3, -1, &submitted, &editing, parts, &root, &owner);
        check(clicked == -1 && editing == (frame == 1), "Path bar must report active editing only while it accepts text.");
        check(frame == 2 ? submitted && strcmp(submitted, "project/source") == 0 : submitted == NULL,
            "Path bar must report submitted text when Enter ends editing.");
        cui_end_frame(ui);
    }
    cui_destroy(ui);
}

int main(void) {
    markdown();
    editor();
    diff();
    hex();
    terminal();
    terminal_rendering();
    terminal_shortcuts();
    log_console();
    source_transitions();
    path_bar();
    path_editing();
    return failures ? 1 : 0;
}
