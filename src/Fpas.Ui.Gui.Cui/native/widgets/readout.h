#ifndef FPAS_CUI_READOUT_H
#define FPAS_CUI_READOUT_H

CUI_API void fpas_cui_readout(cui_ctx *ui, const char *id, const char *label, const char *value,
    const char *unit, const char *detail, int size, int tone, int accent, int align,
    int width_kind, float width);

#endif
