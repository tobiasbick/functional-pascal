#include "../../fpas_cui.h"

void fpas_cui_presence_pill(cui_ctx *ui, const char *id, const cui_live_user *people, int count,
    float size, int max_visible) {
    const cui_presence_pill_opts options = { .size = size, .max_visible = max_visible };
    cui_presence_pill_ex(ui, id, people, count, &options);
}
