#ifndef FPAS_CUI_PROPERTY_INSPECTOR_H
#define FPAS_CUI_PROPERTY_INSPECTOR_H
/** One model-owned property row; changed reports whether CUI proposes an edit. */
typedef struct {
    const char *label, *suffix, *placeholder, *tooltip; /**< Borrowed literal row text, valid during the call. */
    char *text; /**< Writable NUL-terminated UTF-8 text for kind two. */
    const char *const *choices; /**< Borrowed enum captions for kind four. */
    int kind; /**< Float zero, integer one, text two, boolean three, enum four, color five, vector six. */
    int integer; /**< Model integer, boolean or selected enum index; receives proposal. */
    int count; /**< Enum choice count or vector component count (two through four). */
    int capacity; /**< Text buffer bytes including the terminator. */
    int digits; /**< Decimal places zero through nine, or minus one for native formatting. */
    int flags; /**< Enabled bit one and boolean-checkbox style bit two. */
    int changed; /**< Nonzero when the row proposes an edit. */
    float minimum, maximum, step, values[4]; /**< Numeric bounds, step and writable scalar/vector components. */
    uint32_t rgba; /**< Packed RGBA color, updated by a color proposal. */
} fpas_property;
/** Ordered inspector group and its model-owned expansion proposal. */
typedef struct {
    const char *title; /**< Borrowed literal title, valid during the call. */
    fpas_property *properties; /**< Writable rows with count elements. */
    int count, open, collapsible, changed; /**< Row count, expansion, allow-collapse and expansion-changed flag. */
} fpas_property_group;
/** Draws grouped properties and writes proposals into their rows.
 * @param ui Current UI-thread frame context.
 * @param id Stable widget identity.
 * @param groups Writable groups and rows, valid through the call; changed fields identify proposals.
 * @param count Number of groups.
 * @param label_width Initial logical label-column width, or zero for the native default.
 * @param width_kind Extent kind: content zero, fill one, fixed two, percentage three.
 * @param width Logical fixed width or percentage, according to width_kind.
 * @param enabled Whether this widget accepts input.
 */
CUI_API void fpas_cui_property_inspector(cui_ctx *ui, const char *id, fpas_property_group *groups, int count, float label_width, int width_kind, float width, int enabled);
#endif
