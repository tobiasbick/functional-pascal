// [Documentation](../../../../docs/ui/gui/voice/README.md)
#include "../fpas_cui.h"
#include <stdio.h>
#include <string.h>

static bool render_composer(cui_ctx *ui, char *draft, int flags, int focus, int *nodes) {
    int removed = -1, selected = 0;
    const int events = fpas_cui_composer(ui, "composer", draft, 128, "Message", NULL, 0, NULL, 0, &selected,
        NULL, 0, 0, 0, 0, 0, flags, focus, &removed, nodes);
    return (events & CUI_COMPOSER_SEND) != 0;
}

static bool composer_covers_keyboard(cui_ctx *ui) {
    char draft[128] = "Hello";
    int nodes[6];
    cui_begin_frame(ui, 0.05f);
    render_composer(ui, draft, 1, 0, nodes);
    cui_end_frame(ui);

    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, CUI_MOD_SHIFT);
    cui_input_key(ui, CUI_KEY_ENTER, false, CUI_MOD_SHIFT);
    const bool submitted = render_composer(ui, draft, 1, -1, nodes);
    cui_end_frame(ui);
    if (submitted || !strchr(draft, '\n')) return false;

    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const bool disabled_submitted = render_composer(ui, draft, 0, -1, nodes);
    cui_end_frame(ui);
    if (disabled_submitted) return false;

    cui_begin_frame(ui, 0.05f);
    render_composer(ui, draft, 1, 0, nodes);
    cui_end_frame(ui);
    cui_begin_frame(ui, 0.05f);
    cui_input_key(ui, CUI_KEY_ENTER, true, 0);
    cui_input_key(ui, CUI_KEY_ENTER, false, 0);
    const bool active_submitted = render_composer(ui, draft, 1, -1, nodes);
    cui_end_frame(ui);
    return active_submitted;
}

static bool composer_dispatch(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const char *files[] = {"first.txt", "second.txt"};
    const cui_model models[] = {{"First", "One", NULL, CUI_ICON_STAR}, {"Second", "Two", "New", NULL}};
    const float levels[] = {.2f, .6f, .4f};
    char draft[128] = "";
    int nodes[8], selected = 0;
    const struct { int flags, focus; cui_key key; int event; } steps[] = {
        { 1, 0, CUI_KEY_NONE, 0 }, { 1, -1, CUI_KEY_ENTER, CUI_COMPOSER_SEND },
        { 3, 1, CUI_KEY_NONE, 0 }, { 3, -1, CUI_KEY_ENTER, CUI_COMPOSER_STOP },
        { 1, 4, CUI_KEY_NONE, 0 }, { 1, -1, CUI_KEY_ENTER, CUI_COMPOSER_ATTACH },
        { 1, 7, CUI_KEY_NONE, 0 }, { 1, -1, CUI_KEY_ENTER, CUI_COMPOSER_REMOVE },
        { 1, 5, CUI_KEY_NONE, 0 }, { 1, -1, CUI_KEY_ENTER, 0 },
        { 1, -1, CUI_KEY_DOWN, 0 }, { 1, -1, CUI_KEY_DOWN, 0 },
        { 1, -1, CUI_KEY_ENTER, CUI_COMPOSER_MODEL },
        { 5, 2, CUI_KEY_NONE, 0 }, { 5, -1, CUI_KEY_ENTER, CUI_COMPOSER_RECORD },
        { 13, 3, CUI_KEY_NONE, 0 }, { 13, -1, CUI_KEY_ESCAPE, CUI_COMPOSER_CANCEL },
        { 12, -1, CUI_KEY_ESCAPE, 0 },
        { 13, 1, CUI_KEY_NONE, 0 }, { 13, -1, CUI_KEY_SPACE, CUI_COMPOSER_VOICE }
    };
    bool ok = true;
    for (int i = 0; i < (int)(sizeof steps / sizeof *steps); ++i) {
        const cui_key key = steps[i].key;
        cui_begin_frame(ui, .05f);
        if (i == 1) cui_input_text(ui, "hello");
        if (key != CUI_KEY_NONE) {
            cui_input_key(ui, key, true, 0);
            cui_input_key(ui, key, false, 0);
        }
        int removed = -1;
        const int events = fpas_cui_composer(ui, "composer", draft, sizeof draft, NULL,
            files, 2, models, 2, &selected, levels, 3, 2, 3, 2, 460, steps[i].flags,
            steps[i].focus, &removed, nodes);
        bool valid = events == steps[i].event && removed == (i == 7 ? 1 : -1) && selected == (i >= 12 ? 1 : 0);
        if (i == 1) valid &= strcmp(draft, "hello") == 0;
        if (i == 16 || i == 17) valid &= cui_key_pressed(ui, CUI_KEY_ESCAPE) == (i == 17);
        const bool recording = (steps[i].flags & 8) != 0;
        valid &= (nodes[0] < 0) == recording && (nodes[3] >= 0) == recording &&
            (nodes[4] < 0) == recording && (nodes[5] < 0) == recording && nodes[6] >= 0 && nodes[7] >= 0;
        cui_end_frame(ui);
        if (!valid) {
            fprintf(stderr, "Composer frame %d: event=%d, removed=%d, selected=%d\n", i, events, removed, selected);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

static bool composer_recording_height(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const cui_id key = cui_make_id(ui, "composer");
    const float levels[] = {.2f, .4f, .8f};
    char draft[128] = "First\nSecond\nThird";
    float height = 0;
    bool ok = true;
    for (int frame = 0; frame < 125; ++frame) {
        int selected = 0, removed = -1, nodes[6];
        cui_begin_frame(ui, .05f);
        fpas_cui_composer(ui, "composer", draft, sizeof draft, "", NULL, 0, NULL, 0, &selected,
            levels, 3, frame * .05f, 2, 2, 400, frame > 1 ? 13 : 5, -1, &removed, nodes);
        cui_end_frame(ui);
        const cui_rect bounds = cui_get_rect(ui, key);
        if (frame == 1) height = bounds.h;
        if (frame > 1) ok &= bounds.h == height;
        ok &= bounds.w == 400;
    }
    cui_destroy(ui);
    if (!ok) fprintf(stderr, "Composer height changed while recording a take\n");
    return ok;
}

static bool composer_contract(void) {
    return composer_dispatch() && composer_recording_height();
}

static bool model_menu_selects_with_enter(cui_ctx *ui) {
    const cui_model models[] = { { "First", "One", NULL, NULL }, { "Second", "Two", NULL, NULL } };
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_ENTER, CUI_KEY_DOWN, CUI_KEY_DOWN, CUI_KEY_ENTER };
    int selected = 0;
    for (int frame = 0; frame < 5; ++frame) {
        cui_begin_frame(ui, 0.05f);
        if (keys[frame] != CUI_KEY_NONE) {
            cui_input_key(ui, keys[frame], true, 0);
            cui_input_key(ui, keys[frame], false, 0);
        }
        fpas_cui_model_picker(ui, "models", models, 2, &selected, CUI_PLACE_BOTTOM, frame == 0 ? 3 : 1);
        cui_end_frame(ui);
    }
    return selected == 1;
}

static bool status_stops_when_motion_is_reduced(bool adapter) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, true);
    for (int frame = 0; frame < 12; ++frame) {
        cui_begin_frame(ui, 0.05f);
        const cui_plan_step step = { "Reading", NULL, CUI_TASK_RUNNING };
        int node;
        if (adapter) fpas_cui_agent_plan(ui, "job", &step, 1, NULL, 1, &node);
        else cui_agent_plan(ui, "job", &step, 1);
        cui_end_frame(ui);
    }
    const bool stopped = !cui_needs_redraw(ui);
    cui_destroy(ui);
    return stopped;
}

/** Renders the running status and checks its redraw contract after it settles. */
static bool agent_status_redraw(bool adapter, bool reduced) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 480, 240, 1);
    cui_set_reduced_motion(ui, reduced);
    for (int frame = 0; frame < 4; ++frame) {
        cui_begin_frame(ui, .07f);
        if (adapter) fpas_cui_agent_status(ui, "status", "Reading", CUI_TASK_RUNNING, 65, CUI_ICON_GLOBE);
        else cui_agent_status(ui, "Reading", .status = CUI_TASK_RUNNING, .seconds = 65, .icon = CUI_ICON_GLOBE);
        cui_end_frame(ui);
    }
    const bool redraw = cui_needs_redraw(ui);
    cui_destroy(ui);
    return redraw;
}

typedef struct { bool open, focused; float tail, text_right; } ai_tool_state;

/** Draws the tool and reports the following widget's layout and payload glyphs. */
static ai_tool_state draw_ai_tool(cui_ctx *ui, bool adapter, const char *payload) {
    int header = -1;
    bool open = false;
    cui_begin_frame(ui, .04f);
    if (adapter) open = fpas_cui_tool_call(ui, "tool", "read_file", CUI_TASK_DONE, CUI_ICON_FILE_TEXT,
        "Report", payload, NULL, 2.1f, 1, &header);
    else cui_tool_call(ui, "tool", "read_file", .status = CUI_TASK_DONE, .icon = CUI_ICON_FILE_TEXT,
        .detail = "Report", .input = payload, .seconds = 2.1f);
    cui_button(ui, "Following");
    cui_end_frame(ui);
    const cui_draw_data *draw = cui_get_draw_data(ui);
    const cui_rect head = cui_get_rect(ui, cui_make_id(ui, "tool"));
    const float tail = cui_get_rect(ui, cui_make_id(ui, "Following")).y;
    float text_right = 0;
    for (int i = 0; i < draw->vertex_count; ++i) {
        const cui_vertex *v = &draw->vertices[i];
        if (v->mode == CUI_SHADE_TEXT && v->y > head.y + head.h && v->y < tail && v->x > text_right)
            text_right = v->x;
    }
    return (ai_tool_state){ open, fpas_cui_focused(ui, header), tail, text_right };
}

/** Clicks a rendered tool header and returns the expanded frame. */
static ai_tool_state open_ai_tool(cui_ctx *ui, bool adapter, const char *payload) {
    const cui_rect head = cui_get_rect(ui, cui_make_id(ui, "tool"));
    cui_input_mouse_move(ui, head.x + head.w / 2, head.y + 10);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
    draw_ai_tool(ui, adapter, payload);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
    return draw_ai_tool(ui, adapter, payload);
}

/** Checks string payload rendering and retirement of the open, focused row. */
static bool tool_payload_lifecycle(void) {
    cui_ctx *raw = cui_create(NULL), *bridge = cui_create(NULL);
    cui_set_viewport(raw, 600, 310, 1);
    cui_set_viewport(bridge, 600, 310, 1);
    cui_set_reduced_motion(raw, true);
    cui_set_reduced_motion(bridge, true);
    const char *payload = "  Indented payload\n\t mixed indentation";
    const ai_tool_state raw_closed = draw_ai_tool(raw, false, payload);
    const ai_tool_state bridge_closed = draw_ai_tool(bridge, true, payload);
    const ai_tool_state raw_open = open_ai_tool(raw, false, payload);
    const ai_tool_state bridge_open = open_ai_tool(bridge, true, payload);
    const ai_tool_state raw_absent = draw_ai_tool(raw, false, NULL);
    const ai_tool_state bridge_absent = draw_ai_tool(bridge, true, NULL);
    const ai_tool_state raw_restored = draw_ai_tool(raw, false, payload);
    const ai_tool_state bridge_restored = draw_ai_tool(bridge, true, payload);
    const ai_tool_state raw_spaces = draw_ai_tool(raw, false, "  Indented payload\n     mixed indentation");
    const bool ok = raw_closed.tail == bridge_closed.tail && raw_open.tail == bridge_open.tail &&
        bridge_open.open && bridge_open.focused &&
        bridge_open.text_right == raw_spaces.text_right && raw_open.text_right > raw_spaces.text_right &&
        raw_absent.tail == raw_closed.tail && raw_restored.tail == raw_open.tail &&
        !bridge_absent.open && !bridge_absent.focused && bridge_absent.tail == bridge_closed.tail &&
        !bridge_restored.open && !bridge_restored.focused && bridge_restored.tail == bridge_closed.tail;
    cui_destroy(bridge);
    cui_destroy(raw);
    return ok;
}

/** Checks that a manual reasoning disclosure choice survives later thinking cycles. */
static bool reasoning_lifecycle(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 480, 300, 1);
    cui_set_reduced_motion(ui, true);
    const bool thinking[] = { false, true, false, false, true, false };
    const bool expected[] = { false, true, false, true, true, true };
    bool ok = true;
    for (int i = 0; i < 6; ++i) {
        if (i == 3) {
            cui_input_key(ui, CUI_KEY_ENTER, true, 0);
            cui_input_key(ui, CUI_KEY_ENTER, false, 0);
        }
        cui_begin_frame(ui, .04f);
        int nodes[2];
        const int state = fpas_cui_reasoning_begin(ui, "reasoning", thinking[i], 8,
            "Summary", i == 3 ? 3 : 1, nodes);
        if (state & 1) {
            cui_text(ui, "Reasoning body");
            fpas_cui_reasoning_end(ui, nodes[0], nodes[1]);
        }
        cui_end_frame(ui);
        if (((state & 2) != 0) != expected[i]) ok = false;
    }
    cui_destroy(ui);
    return ok;
}

/** Runs the task status, tool payload, and reasoning regressions. */
static bool ai_task_rendering(void) {
    const bool status = agent_status_redraw(false, true) && !agent_status_redraw(true, true) && agent_status_redraw(true, false);
    const bool payload = tool_payload_lifecycle();
    const bool reasoning = reasoning_lifecycle();
    if (!status) fprintf(stderr, "Agent status motion differs from the selected motion preference.\n");
    if (!payload) fprintf(stderr, "Tool payload display or retirement differs from the contract.\n");
    if (!reasoning) fprintf(stderr, "Reasoning did not retain the user disclosure choice.\n");
    return status && payload && reasoning;
}

static void ai_action_key(cui_ctx *ui, cui_key key) {
    cui_input_key(ui, key, true, 0);
    cui_input_key(ui, key, false, 0);
}

static bool ai_source_is_visible(cui_ctx *ui, cui_id card) {
    const cui_rect bounds = cui_get_rect(ui, card);
    const cui_rect viewport = cui_get_rect(ui, cui_make_id(ui, "sources"));
    return bounds.x >= viewport.x - 1 && bounds.x + bounds.w <= viewport.x + viewport.w + 1;
}

static bool ai_source_keyboard_scroll(void) {
    const cui_source documents[] = {
        { "First", "https://one.test/a", NULL },
        { "Second", "https://two.test/b", NULL },
        { "Third", "https://three.test/c", NULL },
    };
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 240, 230, 1);
    cui_set_reduced_motion(ui, true);
    const cui_key keys[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_NONE, CUI_KEY_ENTER };
    bool ok = true;
    int parts[3];
    for (int frame = 0; frame < 6; ++frame) {
        if (keys[frame] != CUI_KEY_NONE) ai_action_key(ui, keys[frame]);
        cui_begin_frame(ui, .05f);
        const int selected = fpas_cui_source_list(ui, "sources", documents, 3, 1, -1, parts);
        cui_end_frame(ui);
        if (selected != (frame == 5 ? 2 : -1)) ok = false;
    }
    const cui_id focused = fpas_cui_node_key(ui, parts[2]);
    if (!focused || cui_focused_id(ui) != focused || !ai_source_is_visible(ui, focused)) ok = false;

    const int widths[] = { 630, 630, 275, 275 };
    for (int frame = 0; frame < 4; ++frame) {
        cui_set_viewport(ui, widths[frame], 230, 1);
        cui_begin_frame(ui, .05f);
        fpas_cui_source_list(ui, "sources", documents, 3, 1, -1, parts);
        cui_end_frame(ui);
    }
    if (cui_focused_id(ui) != focused || !ai_source_is_visible(ui, focused)) ok = false;
    if (!ok) fprintf(stderr, "Source list did not reveal and activate the focused card.\n");
    cui_destroy(ui);
    return ok;
}

static void ai_record_copy(void *user, const char *text) {
    const char **copied = user;
    *copied = !strcmp(text, "An answer.") ? "An answer." : "";
}

static bool ai_source_tooltip_is_valid(void) {
    char url[260];
    memset(url, 'a', sizeof url);
    memcpy(url, "https://example.test/", 21);
    memcpy(&url[254], "\xc3\xa9" "end", 6);
    const cui_source document = { "Example", url, NULL };
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 250, 1);
    cui_set_reduced_motion(ui, true);
    int node;
    cui_begin_frame(ui, .08f);
    fpas_cui_source_list(ui, "tooltip", &document, 1, 1, -1, &node);
    cui_end_frame(ui);
    const cui_rect card = cui_get_rect(ui, fpas_cui_node_key(ui, node));
    cui_input_mouse_move(ui, card.x + card.w / 2, card.y + card.h / 2);
    for (int i = 0; i < 12; ++i) {
        cui_begin_frame(ui, .08f);
        fpas_cui_source_list(ui, "tooltip", &document, 1, 1, -1, &node);
        cui_end_frame(ui);
    }
    int tooltip;
    const char *text = fpas_cui_tooltip_inspect(ui, &tooltip);
    const bool ok = tooltip >= 0 && text && strlen(text) == 254 && !memcmp(text, url, 254);
    if (!ok) fprintf(stderr, "Source tooltip did not end at a UTF-8 character boundary.\n");
    cui_destroy(ui);
    return ok;
}

static bool ai_response_action_parts(void) {
    const char *copied = NULL;
    const cui_platform_iface platform = { .set_clipboard = ai_record_copy, .user = &copied };
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 230, 1);
    cui_set_reduced_motion(ui, true);
    cui_set_platform(ui, &platform);
    const int action[] = { CUI_RESPONSE_COPY, CUI_RESPONSE_REGENERATE, CUI_RESPONSE_UP, CUI_RESPONSE_DOWN, CUI_RESPONSE_SHARE };
    const int expected_vote[] = { 0, 0, 1, -1, -1 };
    int vote = 0, parts[5];
    bool ok = true;
    for (int i = 0; i < 5; ++i) {
        cui_begin_frame(ui, .05f);
        fpas_cui_response_actions(ui, "actions", "An answer.", &vote, 1, i, parts);
        cui_end_frame(ui);
        if (parts[i] < 0 || cui_focused_id(ui) != fpas_cui_node_key(ui, parts[i])) ok = false;
        ai_action_key(ui, CUI_KEY_ENTER);
        cui_begin_frame(ui, .05f);
        const int pressed = fpas_cui_response_actions(ui, "actions", "An answer.", &vote, 1, -1, parts);
        cui_end_frame(ui);
        if (pressed != action[i] || vote != expected_vote[i]) ok = false;
    }
    if (!copied || strcmp(copied, "An answer.")) ok = false;

    cui_begin_frame(ui, .05f);
    fpas_cui_response_actions(ui, "actions", NULL, &vote, 7, 3, parts);
    cui_end_frame(ui);
    const cui_id down = fpas_cui_node_key(ui, parts[3]);
    if (cui_focused_id(ui) != down || parts[0] >= 0 || parts[1] >= 0 || parts[4] >= 0) ok = false;
    ai_action_key(ui, CUI_KEY_ENTER);
    cui_begin_frame(ui, .05f);
    const int cleared = fpas_cui_response_actions(ui, "actions", NULL, &vote, 7, 0, parts);
    cui_end_frame(ui);
    if (cleared != CUI_RESPONSE_DOWN || vote != 0 || cui_focused_id(ui) != down) ok = false;

    ai_action_key(ui, CUI_KEY_ENTER);
    cui_begin_frame(ui, .05f);
    const int disabled = fpas_cui_response_actions(ui, "actions", NULL, &vote, 6, 3, parts);
    cui_end_frame(ui);
    if (disabled || vote != 0 || cui_focused_id(ui) == down) ok = false;
    if (!ok) fprintf(stderr, "Response action focus, visibility, activation or vote state differed.\n");
    cui_destroy(ui);
    return ok;
}

static bool ai_source_and_response_contract(void) {
    return ai_source_keyboard_scroll() && ai_source_tooltip_is_valid() && ai_response_action_parts();
}

static bool voice_wave_passive_cancels_seek(void) {
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return false;
    cui_set_viewport(ui, 480, 240, 1);
    const float peaks[] = { .3f, .5f };
    cui_rect bounds = {0};
    bool ok = true;
    for (int frame = 0; frame < 7; ++frame) {
        if (frame == 1 || frame == 6) {
            cui_input_mouse_move(ui, bounds.x + bounds.w / 2, bounds.y + bounds.h / 2);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        if (frame == 2 || frame == 5) cui_input_mouse_button(ui, CUI_MOUSE_LEFT, false);
        if (frame == 3) {
            cui_input_mouse_move(ui, bounds.x + bounds.w * .8f, bounds.y + bounds.h + 10);
            cui_input_mouse_button(ui, CUI_MOUSE_LEFT, true);
        }
        float position = .1f;
        const bool passive = frame == 2 || frame == 3;
        cui_begin_frame(ui, .016f);
        const int changed = fpas_cui_voice_wave(ui, "voice", peaks, NULL, 2, passive ? NULL : &position,
            2, 300, 0, 0, 0, 0, 0, 1);
        cui_end_frame(ui);
        bounds = cui_get_rect(ui, cui_make_id(ui, "voice"));
        if (changed != (frame == 1 || frame == 6) || (changed ? position < .4f || position > .6f : position != .1f)) {
            fprintf(stderr, "Voice waveform frame %d: position=%g, changed=%d\n", frame, position, changed);
            ok = false;
        }
    }
    cui_destroy(ui);
    return ok;
}

/** Runs the native AI and voice regressions. */
int main(int argc, char **argv) {
    (void)argv;
    if (argc > 1) return status_stops_when_motion_is_reduced(false) ? 0 : 1;
    cui_ctx *ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool composer = composer_covers_keyboard(ui);
    cui_destroy(ui);
    ui = cui_create(NULL);
    if (!ui) return 1;
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    const bool models = model_menu_selects_with_enter(ui);
    cui_destroy(ui);
    if (!composer) fprintf(stderr, "Composer did not distinguish Enter from Shift+Enter and disabled input.\n");
    if (!models) fprintf(stderr, "The model menu did not commit its highlighted model.\n");
    const bool reduced = status_stops_when_motion_is_reduced(true);
    if (!reduced) fprintf(stderr, "A running status still requested frames under reduced motion.\n");
    const bool seek = voice_wave_passive_cancels_seek();
    const bool task_rendering = ai_task_rendering();
    const bool sources_and_actions = ai_source_and_response_contract();
    const bool composer_options = composer_contract();
    if (!task_rendering) fprintf(stderr, "Unexpected native task rendering or interaction.\n");
    if (!sources_and_actions) fprintf(stderr, "Unexpected AI source or response action behavior.\n");
    if (!composer_options) fprintf(stderr, "Unexpected composer options or part behavior.\n");
    return composer && models && reduced && seek && task_rendering && sources_and_actions && composer_options ? 0 : 1;
}
