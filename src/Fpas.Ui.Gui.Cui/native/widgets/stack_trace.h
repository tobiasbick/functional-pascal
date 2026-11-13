#ifndef FPAS_CUI_STACK_TRACE_H
#define FPAS_CUI_STACK_TRACE_H
CUI_API void fpas_cui_stack_trace(cui_ctx *ui, const char *id, const cui_stack_frame * frames, int frames_count, const char * title, const char * message, const char * language, int width_kind, float width, int enabled, fpas_dev_result * out);
#endif
