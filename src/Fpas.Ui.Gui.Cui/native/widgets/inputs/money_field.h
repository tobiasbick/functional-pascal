#ifndef FPAS_MONEY_FIELD_H
#define FPAS_MONEY_FIELD_H
/** Draws CUI's text input for a managed decimal draft; C never parses an amount.
 * @param ui Current frame context.
 * @param id Stable editor identity.
 * @param text Editable NUL-terminated UTF-8 draft, copied for deferred painting.
 * @param capacity Total writable bytes including the terminator.
 * @param label Optional literal caption.
 * @param symbol Literal currency prefix, empty to omit it.
 * @param code Optional literal currency-code suffix.
 * @param hint Optional supporting text.
 * @param error Optional managed validation error.
 * @param width_kind Content zero, fill one, fixed two or percentage three.
 * @param width Logical width or percentage.
 * @param enabled Whether input is accepted.
 * @param focus Whether to request keyboard focus.
 * @param node Receives the native editable field.
 * @return Changed/submitted flags and bit 4 when the editor owns focus. */
CUI_API int fpas_cui_money_field(cui_ctx *ui, const char *id, char *text, size_t capacity,
    const char *label, const char *symbol, const char *code, const char *hint, const char *error,
    int width_kind, float width, int enabled, int focus, int *node);
#endif
