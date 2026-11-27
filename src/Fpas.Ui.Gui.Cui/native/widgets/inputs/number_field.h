// [Documentation](../../../../../docs/ui/gui/inputs/number-field.md)
#ifndef FPAS_NUMBER_FIELD_H
#define FPAS_NUMBER_FIELD_H

/** Draws a native number field and its step buttons.
 * @param ui Drawing context.
 * @param id Stable editor identity.
 * @param value Model value, updated by accepted input.
 * @param min Inclusive lower bound.
 * @param max Inclusive upper bound.
 * @param label Optional caption.
 * @param suffix Optional unit text.
 * @param step Nonnegative increment. Zero uses one.
 * @param decimals Display precision.
 * @param enabled Whether the field accepts input.
 * @param focus Whether to request editor focus.
 * @param width_kind Native width kind, or zero for the upstream default.
 * @param width Native fixed-width or percentage value.
 * @param nodes Receives the editor, decrease and increase nodes. */
CUI_API void fpas_cui_number_field(cui_ctx *ui, const char *id, double *value, double min, double max,
    const char *label, const char *suffix, double step, int decimals, int enabled, int focus,
    int width_kind, float width, int *nodes);

#endif
