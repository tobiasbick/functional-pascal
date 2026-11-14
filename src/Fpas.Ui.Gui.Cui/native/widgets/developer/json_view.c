#include "../../fpas_cui.h"

static bool fpas_json_digit(char c) { return c >= '0' && c <= '9'; }

static bool fpas_json_hex(char c) {
    return fpas_json_digit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F');
}

/* CUI-027 (docs/future/cui-upstream-bugs.md#cui-027): CUI parses structure
   but accepts malformed escape and number tokens. Check their grammar first. */
static const char *fpas_json_bad_token(const char *json) {
    const char *p = json;
    while (*p) {
        if (*p == '"') {
            ++p;
            while (*p && *p != '"') {
                if (*p++ != '\\') continue;
                const char *escape = p - 1;
                if (!*p) return escape;
                if (*p == 'u') {
                    for (int n = 0; n < 4; ++n) if (!fpas_json_hex(*++p)) return escape;
                } else if (!strchr("\"\\/bfnrt", *p)) return escape;
                ++p;
            }
            if (*p) ++p;
        } else if (*p == '-' || fpas_json_digit(*p)) {
            const char *start = p;
            if (*p == '-') ++p;
            if (!fpas_json_digit(*p)) return start;
            if (*p == '0') {
                if (fpas_json_digit(*++p)) return start;
            } else {
                while (fpas_json_digit(*p)) ++p;
            }
            if (*p == '.') {
                if (!fpas_json_digit(*++p)) return start;
                while (fpas_json_digit(*p)) ++p;
            }
            if (*p == 'e' || *p == 'E') {
                ++p;
                if (*p == '+' || *p == '-') ++p;
                if (!fpas_json_digit(*p)) return start;
                while (fpas_json_digit(*p)) ++p;
            }
        } else if (*p == 'n' || *p == 't' || *p == 'f') {
            const char *word = *p == 'n' ? "null" : *p == 't' ? "true" : "false";
            const size_t size = strlen(word);
            if (strncmp(p, word, size) == 0) p += size;
            else ++p;
        } else ++p;
    }
    return NULL;
}

int fpas_cui_json_view(cui_ctx *ui, const char *id, const char *json, int depth, float height, int enabled) {
    const int node = ui->node_count;
    const cui_id key = cui_make_id(ui, id);
    const cui_json_opts options = { .expand_depth = depth, .max_height = height };
    cui_push_disabled(ui, !enabled);
    const char *bad = fpas_json_bad_token(json);
    const bool valid = cui_json_view_ex(ui, id, bad ? "" : json, &options);
    cui_pop_disabled(ui);
    if (bad) {
        int line = 1, column = 1;
        for (const char *p = json; p < bad; ++p) {
            if (*p == '\n') { ++line; column = 1; }
            else ++column;
        }
        const char *kind = *bad == '\\' ? "invalid string escape" : "invalid number";
        const int label = fpas_label_node(ui, node);
        fpas_text(ui, label, cui__frame_printf(ui, "Invalid JSON: %s (line %d, column %d)", kind, line, column));
    }
    fpas_own(ui, key, key ^ 0x15011u, 0, 0);
    ui->last_node = node;
    ui->last_id = key;
    return valid;
}
