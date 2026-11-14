#ifndef FPAS_SCRUB_FIELD_H
#define FPAS_SCRUB_FIELD_H
/** Draws native scrub/typing modes with a stable retained identity and model reconciliation.
 * node identifies the visible outer field, key owns both interaction modes.
 * @param ui Current frame context.
 * @param id Stable identity independent of the visible caption.
 * @param label Literal caption.
 * @param value Finite model value, updated with a native proposal.
 * @param minimum Inclusive lower bound.
 * @param maximum Inclusive upper bound greater than minimum.
 * @param step Native increment, zero for the range-derived default.
 * @param format Validated floating-point display format.
 * @param suffix Optional literal unit suffix.
 * @param icon Optional native glyph replacing the caption.
 * @param width_kind Content zero, fill one, fixed two or percentage three.
 * @param width Logical width or percentage.
 * @param enabled Whether pointer and keyboard input are accepted.
 * @param focus Whether to request the current interaction mode's focus.
 * @param node Receives the visible field node.
 * @param key Receives the stable retained-state owner. */
CUI_API void fpas_cui_scrub_field(cui_ctx *ui, const char *id, const char *label, float *value,
    float minimum, float maximum, float step, const char *format, const char *suffix, const char *icon,
    int width_kind, float width, int enabled, int focus, int *node, unsigned int *key);
#endif
