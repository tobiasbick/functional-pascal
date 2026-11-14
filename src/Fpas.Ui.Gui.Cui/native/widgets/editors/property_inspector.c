#include "../../fpas_cui.h"

static uint32_t fpas_editor_rgba(cui_color color) {
    return (uint32_t)color.r | (uint32_t)color.g << 8 | (uint32_t)color.b << 16 | (uint32_t)color.a << 24;
}

void fpas_cui_property_inspector(cui_ctx *ui, const char *id, fpas_property_group *groups, int count, float label_width, int width_kind, float width, int enabled) {
    const int root = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    fpas_own(ui, key, key ^ 0x15EC0u, 0, 0);
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
                case 2: row->changed = cui_prop_text_ex(ui, row->label, row->text, (size_t)row->capacity, &options); break;
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
    fpas_creative_node(ui, root, key);
}
