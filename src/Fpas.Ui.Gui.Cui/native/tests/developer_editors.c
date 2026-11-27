// [Documentation](../../../../docs/ui/gui/more-developer-tools/api-explorer.md)
#include "../fpas_cui.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static int failures;

static void check(bool passed, const char *description) {
    if (passed) return;
    fprintf(stderr, "%s\n", description);
    ++failures;
}

static cui_ctx *create(void) {
    cui_ctx *ui = cui_create(NULL);
    cui_set_viewport(ui, 700, 550, 1);
    cui_set_reduced_motion(ui, true);
    return ui;
}

static void key(cui_ctx *ui, cui_key value) {
    cui_input_key(ui, value, true, 0);
    cui_input_key(ui, value, false, 0);
}

static void point(cui_ctx *ui, cui_rect rect, bool down) {
    cui_input_mouse_move(ui, rect.x + rect.w * .5f, rect.y + rect.h * .5f);
    cui_input_mouse_button(ui, CUI_MOUSE_LEFT, down);
}

/** Checks native modifiers, modifier edit detection and diagnostics. */
static void regex(void) {
    cui_ctx *ui = create();
    char pattern[16] = "(a)";
    char sample[16] = "A";
    const cui_key sequence[] = { CUI_KEY_NONE, CUI_KEY_TAB, CUI_KEY_TAB, CUI_KEY_SPACE, CUI_KEY_NONE, CUI_KEY_NONE };
    for (int frame = 0; frame < 6; ++frame) {
        if (sequence[frame] != CUI_KEY_NONE) key(ui, sequence[frame]);
        if (frame == 5) strcpy(pattern, "(");
        fpas_dev_result result;
        cui_begin_frame(ui, .016f);
        const char *error = fpas_cui_regex_tester(ui, "regex", pattern, sizeof pattern, sample, sizeof sample,
            -1, 110, 1, 0, 1, &result);
        cui_end_frame(ui);
        if (frame < 3) check(!error && result.flags && result.index == 0 && result.count == 0,
            "Native regex modifiers must initially be zero and match case.");
        if (frame == 3) check(!error && result.flags && result.index == 1 && result.count == 1 && result.action,
            "Space must change the focused native regex modifier and update the count.");
        if (frame == 4) check(!error && result.flags && result.index == 1 && result.count == 1 && !result.action,
            "Native regex modifiers must persist without an idle edit.");
        if (frame == 5) check(error && *error && !result.flags && !result.count,
            "Invalid regex input must expose its native diagnostic.");
    }
    cui_destroy(ui);
}

/** Returns the pinned CUI tag header ID. */
static cui_id api_tag(cui_ctx *ui, const char *tag) {
    uint32_t hash = 2166136261u ^ 0xA9u;
    for (const unsigned char *p = (const unsigned char *)tag; *p; ++p) {
        hash ^= *p;
        hash *= 16777619u;
    }
    hash ^= hash >> 16;
    hash *= 0x7feb352du;
    hash ^= hash >> 15;
    hash *= 0x846ca68bu;
    hash ^= hash >> 16;
    return cui_make_id(ui, "api") ^ (hash ? hash : 1u) ^ 0x7A6u;
}

static const cui_api_endpoint endpoints[] = {
    { "GET", "/first", "First", "First" },
    { "GET", "/middle", "Middle", "Middle" },
    { "GET", "/last", "Last", "Last" }
};

/** Draws an API explorer with its selection held by CUI. */
static fpas_dev_result api_frame(cui_ctx *ui, int flags) {
    fpas_dev_result result;
    cui_begin_frame(ui, .016f);
    fpas_cui_api_explorer(ui, "api", endpoints, 3, -2, -1, 1, 0, flags, &result);
    cui_end_frame(ui);
    return result;
}

/** Checks folded tags, selection ownership and retirement. */
static void api(void) {
    cui_ctx *ui = create();
    const cui_id explorer = cui_make_id(ui, "api");
    const cui_id middle = api_tag(ui, "Middle");
    api_frame(ui, 1);
    api_frame(ui, 1);
    check(cui_get_rect(ui, explorer).h > 500, "A negative API explorer height must fill its parent.");
    const cui_rect header = cui_get_rect(ui, middle);
    check(header.h > 0, "The API explorer must expose the middle tag header.");
    point(ui, header, true);
    api_frame(ui, 1);
    point(ui, header, false);
    api_frame(ui, 1);
    key(ui, CUI_KEY_DOWN);
    fpas_dev_result result = api_frame(ui, 3);
    check(result.index == 0 && result.flags, "The first Down key must select the first visible endpoint.");
    key(ui, CUI_KEY_DOWN);
    result = api_frame(ui, 1);
    check(result.index == 2 && result.flags, "Down must skip endpoints in the folded tag.");
    key(ui, CUI_KEY_UP);
    result = api_frame(ui, 1);
    check(result.index == 0 && result.flags, "Up must skip endpoints in the folded tag.");
    result = api_frame(ui, 1);
    check(result.index == 0 && !result.flags, "An idle API explorer must retain its selection.");
    fpas_cui_forget(ui, explorer);
    result = api_frame(ui, 1);
    check(result.index == -1 && !result.flags, "Retirement must reset native API explorer selection.");
    cui_destroy(ui);
}

/** Draws one flag in the given environments. */
static fpas_dev_result flags_frame(cui_ctx *ui, cui_feature_flag *flag, int environment_count) {
    const char *const environments[] = { "Development" };
    fpas_dev_result result;
    cui_begin_frame(ui, .016f);
    fpas_cui_feature_flags(ui, "flags", flag, 1, environments, environment_count, 0, 1, 0, 1, &result);
    cui_end_frame(ui);
    return result;
}

/** Checks the rollout anchor after a model change and an empty set of environments. */
static void feature_flags(void) {
    cui_ctx *ui = create();
    const cui_id root = cui_make_id(ui, "flags");
    cui_push_id(ui, "flags");
    cui_push_id_int(ui, 0);
    cui_push_id_int(ui, 0);
    const cui_id anchor = cui_make_id(ui, "25%");
    const cui_id popup = cui_make_id(ui, "share");
    cui_pop_id(ui);
    cui_pop_id(ui);
    cui_pop_id(ui);
    cui_feature_flag flag = { "sample", "Sample feature", { true }, { 25 } };
    flags_frame(ui, &flag, 1);
    flags_frame(ui, &flag, 1);
    const cui_rect share = cui_get_rect(ui, anchor);
    check(share.w > 0, "An enabled feature flag must show a share button.");
    point(ui, share, true);
    flags_frame(ui, &flag, 1);
    point(ui, share, false);
    flags_frame(ui, &flag, 1);
    flags_frame(ui, &flag, 1);
    const cui_rect opened = cui_get_rect(ui, popup);
    check(opened.w > 0, "The feature flag share must open its rollout popover.");
    flag.rollout[0] = 50;
    cui_set_viewport(ui, 900, 550, 1);
    const fpas_dev_result result = flags_frame(ui, &flag, 1);
    flags_frame(ui, &flag, 1);
    const cui_rect updated = cui_get_rect(ui, popup);
    check(updated.w > 0 && fabsf(opened.y - updated.y) < 1 && fabsf(opened.x + 200 - updated.x) < 2,
        "A changed rollout must keep the popover anchored to the share button.");
    check(result.index == -1, "An external rollout change must produce no input edit.");
    flags_frame(ui, &flag, 0);
    flags_frame(ui, &flag, 0);
    cui_push_id(ui, "flags");
    cui_push_id_int(ui, 0);
    cui_push_id_int(ui, 0);
    check(cui_get_rect(ui, root).h > 0 && !cui_popover_is_open(ui, "share"),
        "The flags list must draw and close an absent environment's rollout.");
    cui_pop_id(ui);
    cui_pop_id(ui);
    cui_pop_id(ui);
    cui_destroy(ui);
}

int main(void) {
    regex();
    api();
    feature_flags();
    return failures != 0;
}
