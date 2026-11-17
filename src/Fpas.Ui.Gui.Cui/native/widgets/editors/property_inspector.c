#include "../../fpas_cui.h"

typedef struct { cui_id field; fpas_text_binding text; } fpas_property_text;
typedef struct { fpas_property_text *items; int count, capacity, used; } fpas_property_texts;

static void fpas_property_texts_free(cui_ctx *ui, void *data) {
    fpas_property_texts *texts = data;
    for (int i = 0; i < texts->count; ++i) fpas_text_binding_dispose(ui, &texts->items[i].text);
    cui__free(ui, texts->items);
}

static void fpas_property_texts_retire(cui_ctx *ui, cui_id key) {
    const fpas_property_texts *texts = cui__state_lazy(ui, key ^ 0x15EC7u, sizeof *texts, false);
    if (texts) for (int i = 0; i < texts->count; ++i) fpas_forget_state(ui, texts->items[i].field);
}

static fpas_text_binding *fpas_property_text_binding(cui_ctx *ui, fpas_property_texts *texts, cui_id field) {
    int index = texts->used;
    while (index < texts->count && texts->items[index].field != field) ++index;
    if (index == texts->count) {
        if (texts->count == texts->capacity) {
            texts->capacity = texts->capacity ? texts->capacity * 2 : 4;
            texts->items = cui__realloc(ui, texts->items, (size_t)texts->capacity * sizeof *texts->items);
        }
        texts->items[texts->count++] = (fpas_property_text){.field = field};
    }
    const fpas_property_text next = texts->items[index];
    texts->items[index] = texts->items[texts->used];
    texts->items[texts->used++] = next;
    return &texts->items[texts->used - 1].text;
}

static int fpas_property_text_draw(cui_ctx *ui, fpas_property_texts *texts,
    fpas_property *row, const cui_prop_opts *options) {
    /* Match the native property row's input scope without changing its drawing. */
    cui_push_id(ui, row->label);
    const cui_id field = cui_make_id(ui, "##v");
    cui_pop_id(ui);
    fpas_text_binding *prior = fpas_property_text_binding(ui, texts, field);
    if (prior->value && (prior->capacity != (size_t)row->capacity || strcmp(prior->value, row->text))) {
        cui_edit_state *edit = cui__state_lazy(ui, field, sizeof *edit, false);
        if (edit) fpas_text_reset(ui, edit, (int)strlen(row->text));
    }
    const int changed = cui_prop_text_ex(ui, row->label, row->text, (size_t)row->capacity, options);
    if (!prior->value || strcmp(prior->value, row->text)) {
        fpas_text_binding_dispose(ui, prior);
        prior->length = strlen(row->text);
        prior->value = cui__alloc(ui, prior->length + 1);
        memcpy(prior->value, row->text, prior->length + 1);
    }
    prior->capacity = (size_t)row->capacity;
    return changed;
}

static uint32_t fpas_editor_rgba(cui_color color) {
    return (uint32_t)color.r | (uint32_t)color.g << 8 | (uint32_t)color.b << 16 | (uint32_t)color.a << 24;
}

void fpas_cui_property_inspector(cui_ctx *ui, const char *id, fpas_property_group *groups, int count, float label_width, int width_kind, float width, int enabled) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_id texts_key = key ^ 0x15EC7u;
    fpas_own(ui, key, key ^ 0x15EC0u, texts_key, 0);
    fpas_on_forget(ui, key, fpas_property_texts_retire);
    fpas_property_texts *texts = cui_state(ui, texts_key, sizeof *texts);
    cui__state_set_dtor(ui, texts_key, fpas_property_texts_free);
    texts->used = 0;
    cui_push_disabled(ui, !enabled);
    const cui_inspector_opts settings = { .width = fpas_extent(width_kind, width), .label_width = label_width };
    if (cui_inspector_begin_ex(ui, id, &settings)) {
        for (int i = 0; i < count; ++i) {
            fpas_property_group *group = &groups[i];
            bool open = group->open != 0;
            const bool visible = cui_inspector_group(ui, group->title, group->collapsible ? &open : NULL);
            group->changed = open != (group->open != 0);
            group->open = open;
            if (!visible) continue;
            for (int j = 0; j < group->count; ++j) {
                fpas_property *row = &group->properties[j];
                char format[8];
                if (row->digits >= 0 && row->digits <= 9) snprintf(format, sizeof format, "%%.%df", row->digits);
                const cui_prop_opts options = { .min = row->minimum, .max = row->maximum, .step = row->step,
                    .format = row->digits >= 0 && row->digits <= 9 ? format : NULL, .suffix = row->suffix,
                    .placeholder = row->placeholder, .tooltip = row->tooltip,
                    .disabled = !(row->flags & 1), .checkbox = (row->flags & 2) != 0 };
                switch (row->kind) {
                case 0: row->changed = cui_prop_float_ex(ui, row->label, row->values, &options); break;
                case 1: row->changed = cui_prop_int_ex(ui, row->label, &row->integer, &options); break;
                case 2: row->changed = fpas_property_text_draw(ui, texts, row, &options); break;
                case 3: {
                    bool value = row->integer != 0;
                    row->changed = cui_prop_bool_ex(ui, row->label, &value, &options);
                    row->integer = value;
                    break;
                }
                case 4: row->changed = cui_prop_enum_ex(ui, row->label, row->choices, row->count, &row->integer, &options); break;
                case 5: {
                    cui_color value = fpas_chart_color(-1, row->rgba);
                    row->changed = cui_prop_color_ex(ui, row->label, &value, &options);
                    row->rgba = fpas_editor_rgba(value);
                    break;
                }
                case 6: row->changed = cui_prop_vec_ex(ui, row->label, row->values, row->count, &options); break;
                }
            }
        }
        cui_inspector_end(ui);
    }
    cui_pop_disabled(ui);
    while (texts->count > texts->used) {
        fpas_property_text *text = &texts->items[--texts->count];
        fpas_forget_state(ui, text->field);
        fpas_text_binding_dispose(ui, &text->text);
    }
    fpas_creative_node(ui, root, key);
}
