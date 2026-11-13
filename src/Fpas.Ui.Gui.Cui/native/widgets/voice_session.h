#ifndef FPAS_CUI_VOICE_SESSION_H
#define FPAS_CUI_VOICE_SESSION_H
CUI_API int fpas_cui_voice_session(cui_ctx *ui, const char *id, int activity, const float *levels, const uint8_t *owners,
    int count, const char *label, const char *transcript, const char *upcoming, int speaker, const char *you,
    const char *assistant, int *muted, int width_kind, float width, int flags, int focus, int *nodes);
#endif
