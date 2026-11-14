#ifndef FPAS_CUI_PRESENCE_H
#define FPAS_CUI_PRESENCE_H
typedef struct { const char *name, *avatar; int palette; uint32_t rgba; int idle; } fpas_presence_user;
CUI_API void fpas_cui_presence(cui_ctx *ui, const char *id, const fpas_presence_user *users,
    int count, int maximum, float size);
#endif
