/* Model replacements must invalidate a property text editor's old undo history. */
#include "../fpas_cui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char text[128];
    int capacity;
    int changed;
    bool removed;
    unsigned int key;
} property_model;

/** Draws one editable, non-collapsible text property and captures its proposal. */
static void draw(cui_ctx *ui, property_model *model) {
    fpas_property row = { .label = "Name", .kind = 2, .text = model->text,
        .capacity = model->capacity, .digits = -1, .flags = 1 };
    fpas_property_group group = { .title = "Object", .properties = &row,
        .count = model->removed ? 0 : 1, .open = 1 };
    cui_begin_frame(ui, .04f);
    fpas_cui_property_inspector(ui, "properties", &group, 1, 0, 1, 0, 1);
    model->changed = row.changed;
    model->key = fpas_cui_node_key(ui, fpas_cui_last_node(ui));
    cui_end_frame(ui);
}

/** Queues a complete native key stroke for the next frame. */
static void press(cui_ctx *ui, cui_key key, int mods) {
    cui_input_key(ui, key, true, mods);
    cui_input_key(ui, key, false, mods);
}

typedef struct { size_t blocks; } allocation_count;

/** Counts native allocations so replacement and retirement leaks are observable. */
static void *allocate(size_t bytes, void *user) {
    void *value = malloc(bytes);
    if (value) ++((allocation_count *)user)->blocks;
    return value;
}

/** Preserves the allocation count across nonzero native resizes. */
static void *resize(void *value, size_t bytes, void *user) {
    if (!value) return allocate(bytes, user);
    if (!bytes) { free(value); --((allocation_count *)user)->blocks; return NULL; }
    return realloc(value, bytes);
}

/** Releases one counted native allocation. */
static void release(void *value, void *user) {
    if (value) { free(value); --((allocation_count *)user)->blocks; }
}

/** Checks model/capacity replacement, rejected edits, row removal and explicit retirement. */
static bool replacement(int scenario) {
    allocation_count allocations = {0};
    const cui_ctx_desc desc = { .allocator = {allocate, resize, release, &allocations} };
    cui_ctx *ui = cui_create(&desc);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 320, 1);
    cui_set_reduced_motion(ui, true);
    property_model model = { .text = "Original", .capacity = 128 };
    draw(ui, &model);
    draw(ui, &model);
    press(ui, CUI_KEY_TAB, 0);
    draw(ui, &model);
    press(ui, CUI_KEY_A, CUI_MOD_PRIMARY);
    draw(ui, &model);
    /* Text input must not retain the selection shortcut's primary modifier. */
    cui_input_key(ui, CUI_KEY_NONE, false, 0);
    cui_input_text(ui, "Edited property history");
    draw(ui, &model);
    bool valid = model.changed && strcmp(model.text, "Edited property history") == 0;
    if (!valid) fprintf(stderr, "Property editor setup did not produce a text edit.\n");

    /* Ordinary accepted edits retain working undo and redo before any replacement. */
    press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY);
    draw(ui, &model);
    valid = valid && model.changed && strcmp(model.text, "Original") == 0;
    press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY | CUI_MOD_SHIFT);
    draw(ui, &model);
    valid = valid && model.changed && strcmp(model.text, "Edited property history") == 0;

    const char *external = scenario == 2 || scenario >= 4 ? "Edited property history" :
        scenario == 3 ? "Original" : "\xc3\xa9";
    snprintf(model.text, sizeof model.text, "%s", external);
    model.capacity = scenario == 1 ? 4 : scenario == 2 ? 32 : 128;
    if (scenario == 4) {
        model.removed = true;
        draw(ui, &model);
        model.removed = false;
    } else if (scenario == 5) {
        fpas_cui_forget(ui, model.key);
    }
    memset(model.text + model.capacity, 0x5a, sizeof model.text - (size_t)model.capacity);
    draw(ui, &model);
    valid = valid && !model.changed && strcmp(model.text, external) == 0;
    if (scenario >= 4) {
        /* Retired fields release focus; establish real focus before exercising history. */
        press(ui, CUI_KEY_TAB, 0);
        draw(ui, &model);
        press(ui, CUI_KEY_A, CUI_MOD_PRIMARY);
        draw(ui, &model);
    }
    valid = valid && cui_wants_keyboard(ui);
    for (int i = 0; i < 2; ++i) {
        press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY | (i ? CUI_MOD_SHIFT : 0));
        draw(ui, &model);
        if (model.changed || strcmp(model.text, external) != 0) {
            fprintf(stderr, "Property %s after external replacement returned stale text (scenario=%d).\n",
                i ? "redo" : "undo", scenario);
            valid = false;
        }
        for (size_t byte = (size_t)model.capacity; byte < sizeof model.text; ++byte) {
            if ((unsigned char)model.text[byte] != 0x5a) {
                fprintf(stderr, "Property history wrote beyond the replacement capacity.\n");
                valid = false;
                break;
            }
        }
    }
    cui_destroy(ui);
    if (allocations.blocks) {
        fprintf(stderr, "Property history retained %zu allocations after destroy.\n", allocations.blocks);
        valid = false;
    }
    if (!valid) fprintf(stderr, "Property history scenario %d failed.\n", scenario);
    return valid;
}

typedef struct {
    char text[6][64];
    bool reversed, collapsed;
    int changes;
} property_rows;

/** Draws stable labeled rows while allowing the model to reorder or hide them. */
static void draw_rows(cui_ctx *ui, property_rows *model) {
    static const char *labels[] = {"First", "Second", "Third", "Fourth", "Fifth", "Sixth"};
    fpas_property rows[6];
    for (int i = 0; i < 6; ++i) {
        int index = model->reversed ? 5 - i : i;
        rows[i] = (fpas_property){ .label = labels[index], .kind = 2,
            .text = model->text[index], .capacity = sizeof model->text[index],
            .digits = -1, .flags = 1 };
    }
    fpas_property_group group = { .title = "Rows", .properties = rows,
        .count = 6, .open = !model->collapsed, .collapsible = 1 };
    cui_begin_frame(ui, .04f);
    fpas_cui_property_inspector(ui, "rows", &group, 1, 0, 1, 0, 1);
    model->changes = 0;
    for (int i = 0; i < 6; ++i) model->changes += rows[i].changed != 0;
    cui_end_frame(ui);
}

/** Checks model-row identity through reorder, registry growth and collapsed retirement. */
static bool reordered_rows(void) {
    allocation_count allocations = {0};
    const cui_ctx_desc desc = { .allocator = {allocate, resize, release, &allocations} };
    cui_ctx *ui = cui_create(&desc);
    if (!ui) return false;
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    property_rows model = {0};
    for (int i = 0; i < 6; ++i) snprintf(model.text[i], sizeof model.text[i], "Row %d", i);
    draw_rows(ui, &model);
    draw_rows(ui, &model);
    /* The collapsible group header precedes the first editor in tab order. */
    press(ui, CUI_KEY_TAB, 0);
    draw_rows(ui, &model);
    press(ui, CUI_KEY_TAB, 0);
    draw_rows(ui, &model);
    press(ui, CUI_KEY_A, CUI_MOD_PRIMARY);
    draw_rows(ui, &model);
    cui_input_key(ui, CUI_KEY_NONE, false, 0);
    cui_input_text(ui, "Edited first");
    draw_rows(ui, &model);
    bool valid = model.changes == 1 && strcmp(model.text[0], "Edited first") == 0;
    model.reversed = true;
    draw_rows(ui, &model);
    press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY);
    draw_rows(ui, &model);
    valid = valid && model.changes == 1 && strcmp(model.text[0], "Row 0") == 0;
    press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY | CUI_MOD_SHIFT);
    draw_rows(ui, &model);
    valid = valid && model.changes == 1 && strcmp(model.text[0], "Edited first") == 0;
    for (int i = 1; i < 6; ++i) {
        char expected[16];
        snprintf(expected, sizeof expected, "Row %d", i);
        valid = valid && strcmp(model.text[i], expected) == 0;
    }

    model.collapsed = true;
    draw_rows(ui, &model);
    model.collapsed = false;
    draw_rows(ui, &model);
    draw_rows(ui, &model);
    /* The edited First row is now last, after the header and five other editors. */
    for (int tab = 0; tab < 7; ++tab) {
        press(ui, CUI_KEY_TAB, 0);
        draw_rows(ui, &model);
    }
    valid = valid && cui_wants_keyboard(ui);
    for (int redo = 0; redo < 2; ++redo) {
        press(ui, CUI_KEY_Z, CUI_MOD_PRIMARY | (redo ? CUI_MOD_SHIFT : 0));
        draw_rows(ui, &model);
        valid = valid && model.changes == 0 && strcmp(model.text[0], "Edited first") == 0;
    }
    cui_destroy(ui);
    valid = valid && allocations.blocks == 0;
    if (!valid) fprintf(stderr, "Reordered/collapsed property text rows retained incorrect history or allocations.\n");
    return valid;
}

/** Runs the focused model-replacement and retirement regressions without a window. */
int main(void) {
    bool valid = true;
    for (int scenario = 0; scenario < 6; ++scenario)
        valid = replacement(scenario) && valid;
    valid = reordered_rows() && valid;
    return valid ? 0 : 1;
}
