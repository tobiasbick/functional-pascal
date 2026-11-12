#ifndef FPAS_FEEDBACK_H
#define FPAS_FEEDBACK_H

/** Draws a native alert with stable identity and literal title.
 * description/icon are optional borrowed strings; tone is a cui_tone.
 * dismissible/enabled/focus control the native dismiss action.
 * node receives the outer node or -1, dismiss the optional button or -1,
 * key the retained native alert identity even when hidden.
 * Returns visible bit 1 and newly-dismissed bit 2. */
CUI_API int fpas_cui_alert(cui_ctx *ui, const char *id, const char *title,
    const char *description, const char *icon, int tone, int dismissible,
    int enabled, int focus, int *node, int *dismiss, unsigned int *key);

/** Draws native progress. value is a fraction; tone, indeterminate, height,
 * label and show_value map to CUI options. Host sizing supplies width. */
CUI_API void fpas_cui_progress(cui_ctx *ui, const char *id, float value,
    int tone, int indeterminate, float height, const char *label, int show_value);

/** Draws a native spinner with stable inspection identity and logical diameter. */
CUI_API void fpas_cui_spinner(cui_ctx *ui, const char *id, float size);

/** Draws native capacity usage with stable identity and literal label/unit.
 * used and total are finite nonnegative values; total zero uses CUI's empty track. */
CUI_API void fpas_cui_meter(cui_ctx *ui, const char *id, const char *label,
    float used, float total, const char *unit);

/** Draws native circular progress with stable identity and a literal caption.
 * value is a finite fraction from zero through one; size zero uses CUI's default. */
CUI_API void fpas_cui_progress_ring(cui_ctx *ui, const char *id, const char *label,
    float value, float size);

/** Draws a native activity entry with a stable inspection identity and literal text.
 * Empty time, description and icon map to omitted native options; tone is a
 * cui_tone, and last omits the final connector and trailing space. */
CUI_API void fpas_cui_timeline_item(cui_ctx *ui, const char *id, const char *title,
    const char *time, const char *description, const char *icon, int tone, int last);

/** Draws CUI's shimmer placeholder. Width kind is fill 1, logical pixels 2,
 * or percentage 3. Height zero uses CUI's native default. */
CUI_API void fpas_cui_skeleton(cui_ctx *ui, const char *id, int width_kind,
    float width, float height);
#endif
