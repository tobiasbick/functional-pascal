#ifndef FPAS_CUI_PROPERTY_INSPECTOR_H
#define FPAS_CUI_PROPERTY_INSPECTOR_H
typedef struct {
    const char *label, *suffix, *placeholder, *tooltip;
    char *text;
    const char *const *choices;
    int kind, integer, count, capacity, digits, flags, changed;
    float minimum, maximum, step, values[4];
    uint32_t rgba;
} fpas_property;
typedef struct {
    const char *title;
    fpas_property *properties;
    int count, open, collapsible, changed;
} fpas_property_group;
CUI_API void fpas_cui_property_inspector(cui_ctx *ui, const char *id, fpas_property_group *groups, int count, float label_width, int width_kind, float width, int enabled);
#endif
