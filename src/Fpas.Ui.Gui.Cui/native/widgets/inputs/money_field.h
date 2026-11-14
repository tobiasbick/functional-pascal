#ifndef FPAS_MONEY_FIELD_H
#define FPAS_MONEY_FIELD_H
/** Draws CUI money input and reconciles external or rejected MVU values while focused.
 * decimals is the public precision (zero means no fractional digits).
 * @param ui Current frame context.
 * @param id Stable editor identity.
 * @param value Finite model amount, updated with a representable proposal.
 * @param label Optional literal caption.
 * @param symbol Literal currency prefix, empty to omit it.
 * @param code Optional literal currency-code suffix.
 * @param hint Optional supporting text.
 * @param decimals Fractional precision from zero through fifteen.
 * @param width_kind Content zero, fill one, fixed two or percentage three.
 * @param width Logical width or percentage.
 * @param enabled Whether input is accepted.
 * @param focus Whether to request keyboard focus.
 * @param node Receives the native editable field. */
CUI_API void fpas_cui_money_field(cui_ctx *ui, const char *id, double *value,
    const char *label, const char *symbol, const char *code, const char *hint, int decimals,
    int width_kind, float width, int enabled, int focus, int *node);
#endif
