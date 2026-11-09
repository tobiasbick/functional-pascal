/* Text fields: single line input, password, search, multi-line text area and
 * number field. Supports selection with mouse and keyboard, word jumps,
 * clipboard, undo/redo, placeholder, clear button and IME positioning. */
#include "cui_internal.h"


/* cui_edit_state and CUI__UNDO_MAX live in cui_internal.h (the code editor shares them) */

typedef struct {
    const char *buf;
    int len;
    int cursor, anchor;
    bool focused, password, multiline;
    float scroll_x, scroll_y;
    double caret_phase;
    cui_font font;
    float size;
    const char *placeholder;
    cui_color text, placeholder_color, caret, selection;
    float line_h;
    int line_count;
    cui_text_layout_line *lines;
} cui__edit_paint;

static int cui__edit_mask(char *out, int cap, int len_cp) {
    /* bullet per code point */
    int n = 0;
    for (int i = 0; i < len_cp && n + 3 < cap; i++) { memcpy(out + n, "\xE2\x80\xA2", 3); n += 3; }
    out[n] = 0;
    return n;
}

static int cui__cp_count(const char *s, int len) {
    int n = 0;
    for (int i = 0; i < len; i++) if (((unsigned char)s[i] & 0xC0) != 0x80) n++;
    return n;
}


static float cui__x_at(cui_ctx *ui, const cui__edit_paint *p, const char *line, int line_len, int off) {
    if (p->password) {
        char tmp[1024];
        int cps = cui__cp_count(line, off);
        int n = cui__edit_mask(tmp, sizeof tmp, cps);
        return cui__text_width(ui, p->font, p->size, tmp, n, false);
    }
    (void)line_len;
    return cui__text_width(ui, p->font, p->size, line, off, false);
}

static void cui__paint_edit(cui_ctx *ui, cui_node *n, void *data) {
    bool unshaped = ui->text_unshaped;
    cui__edit_paint *p = data;
    cui_rect r = n->rect;
    cui_push_clip(ui, r);
    float fh = cui__font_line_height(ui, p->font, p->size);
    int s0 = CUI_MIN(p->cursor, p->anchor), s1 = CUI_MAX(p->cursor, p->anchor);
    if (p->len == 0 && p->placeholder) {   /* shaped like any label: no caret goes in it */
        float y = r.y + (p->multiline ? (p->line_h - fh) * 0.5f : (r.h - fh) * 0.5f);
        /* A placeholder is static display text; editing retains its original path. */
        ui->text_unshaped = unshaped;
#ifdef CUI_TEXT_SHAPING
        cui_direction direction = ui->text_direction;
        ui->text_direction = n->rtl ? CUI_DIRECTION_RTL : CUI_DIRECTION_LTR;
#endif
        float w = cui__text_width(ui, p->font, p->size, p->placeholder, -1, false);
        float x = n->rtl ? r.x + CUI_MAX(0, r.w - w) : r.x;
        cui__draw_text_ellipsis(ui, p->font, p->size, x, y, r.w, p->placeholder_color, p->placeholder, (int)strlen(p->placeholder), false);
#ifdef CUI_TEXT_SHAPING
        ui->text_direction = direction;
#endif
        ui->text_unshaped = true;
    }
    ui->text_unshaped = true;   /* the text being edited: caret positions follow the glyphs one by one */
    if (!p->multiline) {
        float y = r.y + (r.h - fh) * 0.5f;
        float x = r.x - p->scroll_x;
        const char *shown = p->buf;
        int shown_len = p->len;
        char *mask = NULL;
        if (p->password) {
            mask = cui__frame_alloc(ui, (size_t)p->len * 3 + 4);
            shown_len = cui__edit_mask(mask, p->len * 3 + 4, cui__cp_count(p->buf, p->len));
            shown = mask;
        }
        if (p->focused && s1 > s0) {
            float a = cui__x_at(ui, p, p->buf, p->len, s0), b = cui__x_at(ui, p, p->buf, p->len, s1);
            cui_rect sr = { x + a, r.y + (r.h - fh) * 0.5f - 1, b - a, fh + 2 };
            cui_draw_rect(ui, sr, CUI_CORNERS(3), p->selection);
        }
        cui__draw_text_line(ui, p->font, p->size, x, y, p->text, shown, shown_len, false);
        if (p->focused && s1 == s0) {
            float blink = 0.5f + 0.5f * cosf((float)p->caret_phase * 2.0f * CUI_PI / 1.06f);
            float a = blink > 0.35f ? 1.0f : 0.0f;
            if (p->caret_phase < 0.5) a = 1;
            float cx = x + cui__x_at(ui, p, p->buf, p->len, p->cursor);
            if (p->cursor == 0) cx -= 1.0f;   /* stand clear of the first glyph or the placeholder */
            cui_rect cr = { roundf(cx * ui->scale) / ui->scale - 0.5f, r.y + (r.h - fh) * 0.5f, 1.5f, fh };
            cui_draw_rect(ui, cr, CUI_CORNERS(0.75f), cui_alpha(p->caret, a));
        }
    } else {
        float y0 = r.y - p->scroll_y;
        for (int k = 0; k < p->line_count; k++) {
            cui_text_layout_line *L = &p->lines[k];
            float ly = y0 + p->line_h * (float)k;
            if (ly > r.y + r.h || ly + p->line_h < r.y) continue;
            float ty = ly + (p->line_h - fh) * 0.5f;
            const char *line = p->buf + L->start;
            int ll = L->end - L->start;
            if (p->focused && s1 > s0) {
                int a0 = CUI_MAX(s0, L->start), a1 = CUI_MIN(s1, L->end);
                bool extends = s1 > L->end && k < p->line_count - 1;
                if (a1 > a0 || (extends && a0 <= L->end && s0 <= L->end)) {
                    float xa = cui__text_width(ui, p->font, p->size, line, a0 - L->start, false);
                    float xb = cui__text_width(ui, p->font, p->size, line, CUI_MAX(a1, a0) - L->start, false) + (extends ? 4 : 0);
                    cui_rect sr = { r.x + xa, ly, xb - xa, p->line_h };
                    cui_draw_rect(ui, sr, CUI_CORNERS(2), p->selection);
                }
            }
            cui__draw_text_line(ui, p->font, p->size, r.x, ty, p->text, line, ll, false);
            if (p->focused && s0 == s1) {
                bool here = p->cursor >= L->start && (p->cursor < L->end || (p->cursor == L->end && (k == p->line_count - 1 || p->lines[k + 1].start > L->end)));
                if (k + 1 < p->line_count && p->cursor == p->lines[k + 1].start && p->cursor == L->end) here = false;
                if (here) {
                    float blink = 0.5f + 0.5f * cosf((float)p->caret_phase * 2.0f * CUI_PI / 1.06f);
                    float a = (blink > 0.35f || p->caret_phase < 0.5) ? 1.0f : 0.0f;
                    float cx = r.x + cui__text_width(ui, p->font, p->size, line, p->cursor - L->start, false);
                    cui_rect cr = { roundf(cx * ui->scale) / ui->scale - 0.5f, ty, 1.5f, fh };
                    cui_draw_rect(ui, cr, CUI_CORNERS(0.75f), cui_alpha(p->caret, a));
                }
            }
        }
    }
    cui_pop_clip(ui);
    ui->text_unshaped = unshaped;
}

/* ---- editing primitives ---- */

static bool cui__is_word(unsigned char c) { return c >= 128 || (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'; }

static int cui__word_left(const char *s, int pos) {
    while (pos > 0 && !cui__is_word((unsigned char)s[pos - 1])) pos--;
    while (pos > 0 && cui__is_word((unsigned char)s[pos - 1])) pos--;
    return pos;
}

static int cui__word_right(const char *s, int len, int pos) {
    while (pos < len && !cui__is_word((unsigned char)s[pos])) pos++;
    while (pos < len && cui__is_word((unsigned char)s[pos])) pos++;
    return pos;
}

/* kind: 1 insert, 2 delete, 3 other. Consecutive edits of the same kind at
 * the caret within a short time merge into one undo step. */
static void cui__undo_push_kind(cui_ctx *ui, cui_edit_state *e, const char *buf, int kind) {
    if (ui->time - e->last_edit < 1.0 && e->undo_pos > 0 && e->undo_pos == e->undo_count && kind == e->last_kind &&
        kind != 3 && e->cursor == e->last_cursor) {
        e->last_edit = ui->time;
        return;
    }
    e->last_edit = ui->time;
    e->last_kind = kind;
    for (int i = e->undo_pos; i < e->undo_count; i++) { cui__free(ui, e->undo[i]); e->undo[i] = NULL; }
    e->undo_count = e->undo_pos;
    if (e->undo_count == CUI__UNDO_MAX) {
        cui__free(ui, e->undo[0]);
        memmove(e->undo, e->undo + 1, sizeof(char *) * (CUI__UNDO_MAX - 1));
        memmove(e->undo_cursor, e->undo_cursor + 1, sizeof(int) * (CUI__UNDO_MAX - 1));
        e->undo_count--;
    }
    size_t n = strlen(buf);
    e->undo[e->undo_count] = cui__alloc(ui, n + 1);
    memcpy(e->undo[e->undo_count], buf, n + 1);
    e->undo_cursor[e->undo_count] = e->cursor;
    e->undo_count++;
    e->undo_pos = e->undo_count;
}

static void cui__undo_push(cui_ctx *ui, cui_edit_state *e, const char *buf) { cui__undo_push_kind(ui, e, buf, 3); }

/* One step back (or forward with redo) in the history; false when there is
 * none. The caret lands where it was before the edit that is undone. */
bool cui__edit_undo_step(cui_ctx *ui, cui_edit_state *e, char *buf, size_t cap, bool redo) {
    if (!redo) {
        if (e->undo_pos == e->undo_count) { cui__undo_push(ui, e, buf); e->undo_pos--; e->last_edit = 0; }
        if (e->undo_pos <= 0) return false;
        e->undo_pos--;
    } else {
        if (e->undo_pos + 1 >= e->undo_count) return false;
        e->undo_pos++;
    }
    snprintf(buf, cap, "%s", e->undo[e->undo_pos]);
    int len = (int)strlen(buf);
    e->cursor = e->anchor = CUI_MIN(e->undo_cursor[e->undo_pos], len);
    return true;
}

static void cui__delete_range(char *buf, int *len, int a, int b) {
    if (b <= a) return;
    memmove(buf + a, buf + b, (size_t)(*len - b + 1));
    *len -= b - a;
}

static int cui__insert(char *buf, int *len, size_t cap, int pos, const char *s, int n, bool multiline) {
    /* filtered length: carriage returns dropped; newlines and tabs become
     * spaces in single line fields */
    int m = 0;
    for (int i = 0; i < n; i++) if (s[i] != '\r') m++;
    int room = (int)cap - 1 - *len;
    if (m > room) m = room;
    if (m <= 0) return 0;
    /* never cut a UTF-8 sequence: find how many source bytes fit, then back off */
    int take = 0, kept = 0;
    while (take < n && kept < m) { if (s[take] != '\r') kept++; take++; }
    while (take < n && take > 0 && ((unsigned char)s[take] & 0xC0) == 0x80) {
        /* the next byte continues a sequence we would split: drop that sequence */
        take--;
        if (s[take] != '\r') kept--;
        while (take > 0 && ((unsigned char)s[take] & 0xC0) == 0x80) { take--; if (s[take] != '\r') kept--; }
    }
    m = kept;
    if (m <= 0) return 0;
    memmove(buf + pos + m, buf + pos, (size_t)(*len - pos + 1));
    int o = pos;
    for (int i = 0; i < take; i++) {
        char c = s[i];
        if (c == '\r') continue;
        if (!multiline && (c == '\n' || c == '\t')) c = ' ';
        buf[o++] = c;
    }
    *len += m;
    return m;
}

/* Lines for a multi-line layout at the given width. */
int cui__edit_lines(cui_ctx *ui, cui_font f, float size, const char *buf, int len, float width, cui_text_layout_line **out) {
    if (width <= 0) width = 1;
    int n = cui__text_wrap(ui, f, size, buf, len, width, NULL, 0);
    if (n < 1) n = 1;
    cui_text_layout_line *lines = cui__frame_alloc(ui, sizeof(cui_text_layout_line) * (size_t)(n + 1));
    cui__text_wrap(ui, f, size, buf, len, width, lines, n);
    /* text wrap trims trailing spaces; keep lines contiguous for caret math */
    for (int k = 0; k < n - 1; k++) if (lines[k].end < lines[k + 1].start && buf[lines[k].end] != '\n') lines[k].end = lines[k + 1].start;
    if (len > 0 && buf[len - 1] == '\n') {
        lines[n].start = lines[n].end = len; lines[n].width = 0;
        n++;
    }
    if (n > 0) lines[n - 1].end = len;
    *out = lines;
    return n;
}

static int cui__line_of(const cui_text_layout_line *lines, int n, int pos) {
    for (int k = 0; k < n; k++) {
        if (pos < lines[k].end) return k;
        if (pos == lines[k].end && (k == n - 1 || lines[k + 1].start > pos)) return k;
    }
    return n - 1;
}

typedef struct { bool focused, changed, submitted, blurred, gained; } cui__edit_result;

/* One frame of editing a focused field: what the phases below share. */
typedef struct {
    cui_ctx *ui;
    cui_edit_state *e;
    char *buf; size_t cap; int len;
    bool multiline, password, read_only;
    cui_rect area;                       /* the text area, last frame */
    cui_font font; float size, line_h;
    cui_text_layout_line *lines; int nlines;   /* wrapped lines, text areas only */
    cui__edit_paint measure;             /* font, size and masking, for cui__x_at */
    bool shift, word_mod, line_mod, prim;
    bool moved, edited, vertical;
} cui__edit_frame;

/* the caret goes to `to`; without Shift the selection collapses onto it */
static void cui__edit_move(cui__edit_frame *c, int to) {
    c->e->cursor = to;
    if (!c->shift) c->e->anchor = to;
    c->moved = true;
}

/* replace the selection with `text` (n bytes) and put the caret after it */
static void cui__edit_replace_selection(cui__edit_frame *c, const char *text, int n, bool newlines) {
    cui_edit_state *e = c->e;
    int s0 = CUI_MIN(e->cursor, e->anchor), s1 = CUI_MAX(e->cursor, e->anchor);
    cui__delete_range(c->buf, &c->len, s0, s1);
    e->cursor = e->anchor = s0;
    int m = cui__insert(c->buf, &c->len, c->cap, e->cursor, text, n, newlines);
    e->cursor += m; e->anchor = e->cursor;
    c->edited = true;
}

/* the pointer: place the caret, select by dragging, words on a double click, all on a triple */
static void cui__edit_pointer(cui__edit_frame *c, cui_interact r) {
    cui_ctx *ui = c->ui;
    cui_edit_state *e = c->e;
    const char *buf = c->buf;
    int len = c->len;
    if (!(r.active && (ui->mdown[0] || ui->mpressed[0]) && c->area.w > 0)) return;
    int pos;
    if (!c->multiline) {
        float x = ui->mx - c->area.x + e->scroll_x;
        if (c->password) {
            /* walk code points */
            int p = 0, best = 0;
            float bestd = 1e9f;
            for (;;) {
                float px = cui__x_at(ui, &c->measure, buf, len, p);
                float dd = fabsf(px - x);
                if (dd < bestd) { bestd = dd; best = p; }
                if (p >= len) break;
                p = cui__utf8_next(buf, len, p);
            }
            pos = best;
        } else pos = cui__text_hit(ui, c->font, c->size, buf, len, x);
    } else {
        float y = ui->my - c->area.y + e->scroll_y;
        int ln = (int)floorf(y / c->line_h);
        ln = CUI_CLAMP(ln, 0, c->nlines - 1);
        cui_text_layout_line *L = &c->lines[ln];
        pos = L->start + cui__text_hit(ui, c->font, c->size, buf + L->start, L->end - L->start, ui->mx - c->area.x);
        if (pos > L->end) pos = L->end;
        if (ln < c->nlines - 1 && pos == L->end && L->end == c->lines[ln + 1].start && pos > L->start) pos = cui__utf8_prev(buf, pos);
    }
    if (ui->mpressed[0]) {
        if (r.double_clicked || ui->click_count >= 2) {
            e->anchor = cui__word_left(buf, pos);
            if (pos < len && cui__is_word((unsigned char)buf[pos])) e->anchor = cui__word_left(buf, pos + 1);
            e->cursor = cui__word_right(buf, len, pos);
            if (ui->click_count >= 3) { e->anchor = 0; e->cursor = len; }
        } else {
            e->cursor = pos;
            if (!(ui->mods & CUI_MOD_SHIFT)) e->anchor = pos;
        }
        e->selecting = true;
    } else if (e->selecting && ui->click_count < 2) {
        e->cursor = pos;
    }
    ui->caret_time = ui->time;
    e->preferred_x = -1;
}

/* arrows, Home, End and select all */
static void cui__edit_move_keys(cui__edit_frame *c) {
    cui_ctx *ui = c->ui;
    cui_edit_state *e = c->e;
    const char *buf = c->buf;
    int len = c->len;
    bool shift = c->shift, word_mod = c->word_mod, line_mod = c->line_mod;
    int s0 = CUI_MIN(e->cursor, e->anchor), s1 = CUI_MAX(e->cursor, e->anchor);
    for (int rep = cui_key_pressed(ui, CUI_KEY_LEFT) ? ui->kpressed[CUI_KEY_LEFT] : 0; rep > 0; rep--) {
        if (!shift && s1 > s0 && !word_mod) { e->cursor = e->anchor = s0; c->moved = true; continue; }
        cui__edit_move(c, line_mod ? 0 : word_mod ? cui__word_left(buf, e->cursor) : cui__utf8_prev(buf, e->cursor));
    }
    for (int rep = cui_key_pressed(ui, CUI_KEY_RIGHT) ? ui->kpressed[CUI_KEY_RIGHT] : 0; rep > 0; rep--) {
        if (!shift && s1 > s0 && !word_mod) { e->cursor = e->anchor = s1; c->moved = true; continue; }
        cui__edit_move(c, line_mod ? len : word_mod ? cui__word_right(buf, len, e->cursor) : cui__utf8_next(buf, len, e->cursor));
    }
    c->vertical = c->multiline && (cui_key_pressed(ui, CUI_KEY_UP) || cui_key_pressed(ui, CUI_KEY_DOWN));
    if (c->vertical) {
        cui_text_layout_line *lines = c->lines;
        int nlines = c->nlines;
        int ln = cui__line_of(lines, nlines, e->cursor);
        cui_text_layout_line *L = &lines[ln];
        float x = e->preferred_x >= 0 ? e->preferred_x : cui__text_width(ui, c->font, c->size, buf + L->start, e->cursor - L->start, false);
        int dir = cui_key_pressed(ui, CUI_KEY_UP) ? -1 : 1;
        int target = ln + dir;
        if (target < 0) cui__edit_move(c, 0);
        else if (target >= nlines) cui__edit_move(c, len);
        else {
            cui_text_layout_line *T = &lines[target];
            int pos = T->start + cui__text_hit(ui, c->font, c->size, buf + T->start, T->end - T->start, x);
            if (target < nlines - 1 && pos == T->end && T->end == lines[target + 1].start && pos > T->start) pos = cui__utf8_prev(buf, pos);
            cui__edit_move(c, pos);
        }
        e->preferred_x = x;
    }
    if (cui_key_pressed(ui, CUI_KEY_HOME)) {
        if (c->multiline && !c->prim) { int ln = cui__line_of(c->lines, c->nlines, e->cursor); cui__edit_move(c, c->lines[ln].start); }
        else cui__edit_move(c, 0);
    }
    if (cui_key_pressed(ui, CUI_KEY_END)) {
        if (c->multiline && !c->prim) {
            cui_text_layout_line *lines = c->lines;
            int ln = cui__line_of(lines, c->nlines, e->cursor);
            int end = lines[ln].end;
            if (ln < c->nlines - 1 && end == lines[ln + 1].start && end > lines[ln].start && buf[end - 1] == ' ') end--;
            cui__edit_move(c, end);
        } else cui__edit_move(c, len);
    }
    if (c->prim && cui_key_pressed(ui, CUI_KEY_A)) { e->anchor = 0; e->cursor = len; c->moved = true; }
}

/* copy, cut and paste */
static void cui__edit_clipboard(cui__edit_frame *c) {
    cui_ctx *ui = c->ui;
    cui_edit_state *e = c->e;
    int s0 = CUI_MIN(e->cursor, e->anchor), s1 = CUI_MAX(e->cursor, e->anchor);
    if (c->prim && (cui_key_pressed(ui, CUI_KEY_C) || cui_key_pressed(ui, CUI_KEY_X)) && s1 > s0 && !c->password) {
        if (ui->platform.set_clipboard) {
            char *tmpc = cui__frame_strdup(ui, c->buf + s0, s1 - s0);
            ui->platform.set_clipboard(ui->platform.user, tmpc);
        }
        if (cui_key_pressed(ui, CUI_KEY_X) && !c->read_only) {
            cui__undo_push(ui, e, c->buf);
            cui__delete_range(c->buf, &c->len, s0, s1);
            e->cursor = e->anchor = s0;   /* the selection is gone; a paste in the same frame inserts at the caret */
            c->edited = true;
        }
    }
    if (!c->read_only && c->prim && cui_key_pressed(ui, CUI_KEY_V) && ui->platform.get_clipboard) {
        const char *clip = ui->platform.get_clipboard(ui->platform.user);
        if (clip && *clip) {
            cui__undo_push(ui, e, c->buf);
            cui__edit_replace_selection(c, clip, (int)strlen(clip), c->multiline);
            e->last_edit = 0;
        }
    }
}

/* undo and redo, Backspace, Delete, Enter in a text area, and typed text */
static void cui__edit_typing(cui__edit_frame *c, cui__edit_result *res) {
    cui_ctx *ui = c->ui;
    cui_edit_state *e = c->e;
    char *buf = c->buf;
    int s0, s1;
    bool undo = c->prim && cui_key_pressed(ui, CUI_KEY_Z) && !c->shift;
    bool redo = c->prim && ((c->shift && cui_key_pressed(ui, CUI_KEY_Z)) || cui_key_pressed(ui, CUI_KEY_Y));
    if ((undo || redo) && cui__edit_undo_step(ui, e, buf, c->cap, !undo)) {
        c->len = (int)strlen(buf);
        res->changed = true;
    }
    for (int rep = ui->kpressed[CUI_KEY_BACKSPACE]; rep > 0; rep--) {
        s0 = CUI_MIN(e->cursor, e->anchor); s1 = CUI_MAX(e->cursor, e->anchor);
        cui__undo_push_kind(ui, e, buf, s1 > s0 ? 3 : 2);
        if (s1 > s0) { cui__delete_range(buf, &c->len, s0, s1); e->cursor = e->anchor = s0; }
        else if (e->cursor > 0) {
            int to = c->word_mod ? cui__word_left(buf, e->cursor) : c->line_mod ? 0 : cui__utf8_prev(buf, e->cursor);
            cui__delete_range(buf, &c->len, to, e->cursor);
            e->cursor = e->anchor = to;
        }
        c->edited = true;
    }
    for (int rep = ui->kpressed[CUI_KEY_DELETE]; rep > 0; rep--) {
        s0 = CUI_MIN(e->cursor, e->anchor); s1 = CUI_MAX(e->cursor, e->anchor);
        cui__undo_push_kind(ui, e, buf, s1 > s0 ? 3 : 2);
        if (s1 > s0) { cui__delete_range(buf, &c->len, s0, s1); e->cursor = e->anchor = s0; }
        else if (e->cursor < c->len) {
            int to = c->word_mod ? cui__word_right(buf, c->len, e->cursor) : cui__utf8_next(buf, c->len, e->cursor);
            cui__delete_range(buf, &c->len, e->cursor, to);
        }
        c->edited = true;
    }
    if (c->multiline && cui_key_pressed(ui, CUI_KEY_ENTER) && !c->prim) {
        cui__undo_push(ui, e, buf);
        cui__edit_replace_selection(c, "\n", 1, true);
    }
    /* Tab is left to focus navigation, in text areas too */
    if (ui->text_in_len > 0 && !(c->prim && !(ui->mods & CUI_MOD_ALT))) {
        s0 = CUI_MIN(e->cursor, e->anchor); s1 = CUI_MAX(e->cursor, e->anchor);
        cui__undo_push_kind(ui, e, buf, s1 > s0 ? 3 : 1);
        cui__edit_replace_selection(c, ui->text_in, ui->text_in_len, c->multiline);
    }
}

/* scroll so the caret stays in view, and tell the input method where it is */
static void cui__edit_follow_caret(cui__edit_frame *c, const cui__edit_result *res) {
    cui_ctx *ui = c->ui;
    cui_edit_state *e = c->e;
    cui_rect area = c->area;
    int len = c->len = (int)strlen(c->buf);
    if (!c->multiline) {
        float cx = cui__x_at(ui, &c->measure, c->buf, len, e->cursor);
        float full = cui__x_at(ui, &c->measure, c->buf, len, len);
        if (area.w > 0) {
            if (cx - e->scroll_x > area.w - 2) e->scroll_x = cx - area.w + 2;
            if (cx - e->scroll_x < 0) e->scroll_x = cx;
            if (full - e->scroll_x < area.w - 2) e->scroll_x = CUI_MAX(0, full - area.w + 2);
        }
    } else if (area.h > 0) {
        if (res->changed) c->nlines = cui__edit_lines(ui, c->font, c->size, c->buf, len, area.w, &c->lines);
        if (res->changed || c->moved || e->selecting) {   /* follow the caret only when it moved */
            int ln = cui__line_of(c->lines, c->nlines, e->cursor);
            float cy = ln * c->line_h;
            if (cy - e->scroll_y > area.h - c->line_h) e->scroll_y = cy - area.h + c->line_h;
            if (cy - e->scroll_y < 0) e->scroll_y = cy;
        }
        float total = c->nlines * c->line_h;
        if (e->scroll_y > total - area.h) e->scroll_y = CUI_MAX(0, total - area.h);
    }
    if (ui->platform.set_text_input_rect) {
        cui_rect cr = area;
        if (!c->multiline) cr.x = area.x + cui__x_at(ui, &c->measure, c->buf, len, e->cursor) - e->scroll_x;
        cr.w = 1;
        ui->platform.set_text_input_rect(ui->platform.user, cr);
    }
}

static cui__edit_result cui__edit_logic(cui_ctx *ui, cui_id id, cui_edit_state *e, char *buf, size_t cap,
                                        bool multiline, bool password, bool read_only, cui_rect area,
                                        cui_font font, float size, float line_h, cui_interact r) {
    cui__edit_result res;
    memset(&res, 0, sizeof res);
    int len = (int)strlen(buf);
    if (e->cursor > len) e->cursor = len;
    if (e->anchor > len) e->anchor = len;
    bool focused = ui->focus == id;
    if (focused && !e->had_focus) res.gained = true;
    if (!focused && e->had_focus) res.blurred = true;
    e->had_focus = focused;
    if (!focused) {
        e->selecting = false;
        return res;
    }
    res.focused = true;
    ui->edit = e;

    cui__edit_frame c;
    memset(&c, 0, sizeof c);
    c.ui = ui; c.e = e; c.buf = buf; c.cap = cap; c.len = len;
    c.multiline = multiline; c.password = password; c.read_only = read_only;
    c.area = area; c.font = font; c.size = size; c.line_h = line_h;
    c.measure.font = font; c.measure.size = size; c.measure.password = password;
    if (multiline) c.nlines = cui__edit_lines(ui, font, size, buf, len, area.w, &c.lines);

    cui__edit_pointer(&c, r);
    if (!ui->mdown[0]) e->selecting = false;

    int mods = ui->mods;
    c.shift = (mods & CUI_MOD_SHIFT) != 0;
#if defined(__APPLE__)
    c.word_mod = (mods & CUI_MOD_ALT) != 0;     /* Option+arrow moves by word, Command+arrow to the line's end */
    c.line_mod = (mods & CUI_MOD_SUPER) != 0;
#else
    c.word_mod = (mods & CUI_MOD_CTRL) != 0;
    c.line_mod = false;
#endif
    c.prim = (mods & CUI_MOD_PRIMARY) != 0;

    cui__edit_move_keys(&c);
    cui__edit_clipboard(&c);
    if (!read_only) cui__edit_typing(&c, &res);
    if ((!multiline || c.prim) && cui_key_pressed(ui, CUI_KEY_ENTER)) res.submitted = true;
    if (cui_key_pressed(ui, CUI_KEY_ESCAPE)) {
        if (e->cursor != e->anchor) e->anchor = e->cursor;
        else { ui->focus = 0; res.blurred = true; }
    }
    if (c.edited) res.changed = true;
    if (c.edited || c.moved) ui->caret_time = ui->time;
    if (c.edited) e->last_cursor = e->cursor;
    if ((c.edited || c.moved) && !c.vertical) e->preferred_x = -1;

    cui__edit_follow_caret(&c, &res);
    return res;
}

/* ---- field chrome ---- */

typedef enum { CUI__FIELD_BOXED = 0, CUI__FIELD_BARE = 1 } cui__field_style;

int cui__text_field(cui_ctx *ui, cui_id id, char *buf, size_t cap, const cui_input_opts *o, bool multiline, int style, const char *trailing_icon_default);

static void cui__edit_dtor(cui_ctx *ui, void *data) {
    cui_edit_state *e = data;
    for (int i = 0; i < e->undo_count; i++) cui__free(ui, e->undo[i]);
    e->undo_count = 0;
}

int cui__text_field(cui_ctx *ui, cui_id id, char *buf, size_t cap, const cui_input_opts *o, bool multiline, int style, const char *search_icon) {
    cui_input_opts d;
    memset(&d, 0, sizeof d);
    if (!o) o = &d;
    if (!buf || cap == 0) return 0;
    bool unshaped = ui->text_unshaped;
    ui->text_unshaped = true;
    bool disabled = o->disabled || cui_is_disabled(ui);
    cui_edit_state *e = cui_state(ui, id, sizeof *e);
    if (!e->init) {
        e->init = true;
        e->preferred_x = -1;
        cui__state_set_dtor(ui, id, cui__edit_dtor);
        if (o->autofocus) ui->focus = id;
    }
    cui_id area_id = id ^ 0xED17u;
    if (ui->tf_select_all == id && ui->focus == id) {
        e->anchor = 0;
        e->cursor = (int)strlen(buf);
        e->scroll_x = 0;
        ui->tf_select_all = 0;
    }
    cui_push_disabled(ui, disabled);
    /* the text area itself takes focus; clicks on padding also land on the field */
    cui_interact rf = cui__interact(ui, id, CUI_IF_FOCUSABLE | CUI_IF_TEXT | CUI_IF_DRAG);
    cui_pop_disabled(ui);
    if (rf.hovered) cui__set_cursor(ui, CUI_CURSOR_TEXT);
    bool was_focused = ui->focus == id;
    float size = cui__fs(ui, o->size == CUI_SIZE_SM ? ui->theme.text_sm : ui->theme.text_body);
    cui_font font = CUI_FONT_BODY;
    if (ui->tf_override) {   /* inline edit: type that matches the text it replaces */
        font = ui->tf_font;
        size = ui->tf_size;
        ui->tf_override = false;
    }
    float line_h = size * (size > 18 ? 1.3f : 1.5f);
    cui_rect area = cui__prev_rect(ui, area_id);
    bool password = o->password && !e->show_password;
    cui__edit_result er = cui__edit_logic(ui, id, e, buf, cap, multiline, password, o->read_only || disabled, area,
                                          font, size, line_h, rf);
    if (multiline && rf.hovered && ui->scroll_dy != 0 && area.h > 0) {
        /* wheel scrolls the text whether or not the field has focus */
        cui_text_layout_line *wl = NULL;
        int wn = cui__edit_lines(ui, font, size, buf, (int)strlen(buf), area.w, &wl);
        float total = wn * line_h;
        e->scroll_y = cui__clampf(e->scroll_y - ui->scroll_dy * 40, 0, CUI_MAX(0, total - area.h));
        ui->scroll_dy = 0;
    }
    int flags = 0;
    if (er.changed) flags |= CUI_CHANGED;
    if (er.submitted) flags |= CUI_SUBMITTED;
    if (er.gained) flags |= CUI_FOCUSED;
    if (er.blurred) flags |= CUI_BLURRED;
    cui__tween(ui, &e->hover, rf.hovered ? 1.0f : 0.0f, CUI__T_HOVER);
    cui__tween(ui, &e->focus, (ui->focus == id) ? 1.0f : 0.0f, CUI__T_FOCUS);
    (void)was_focused;

    int len = (int)strlen(buf);
    bool has_error = o->error && *o->error;
    float h = multiline ? (o->height > 0 ? o->height : 96) : cui__control_height(ui, o->size);

    int outer = -1;
    if (style == CUI__FIELD_BOXED && (o->label || o->hint || o->error)) {
        outer = cui__node_new(ui, 0);
        ui->nodes[outer].gap = 6;
        ui->nodes[outer].w = o->width.kind ? o->width : CUI_FILL;
        if (multiline && o->height < 0) ui->nodes[outer].h = CUI_FILL;
        cui__open(ui, outer);
        cui__field_label(ui, o->label);
    }
    int box = cui__node_new(ui, id);
    cui_node *bn = &ui->nodes[box];
    bn->dir = CUI_ROW;
    bn->align = multiline ? CUI_ALIGN_START : CUI_ALIGN_CENTER;
    bn->gap = 8;
    bool fill_h = multiline && o->height < 0;
    bn->h = fill_h ? CUI_FILL : CUI_PX(h);
    if (fill_h) bn->min_h = 80;
    bn->w = outer >= 0 ? CUI_FILL : (o->width.kind ? o->width : CUI_FILL);
    bn->min_w = 80;
    if (style == CUI__FIELD_BOXED) {
        float rad = CUI_MIN(ui->theme.radius_control, h * 0.5f);
        bn->radius = CUI_CORNERS(rad);
        bn->pad[1] = bn->pad[3] = CUI__FIELD_PAD;
        if (multiline) { bn->pad[0] = bn->pad[2] = 8; }
        if (o->icon || search_icon) bn->pad[3] = CUI__FIELD_PAD_ICON;
        bn->bg = disabled ? CUI_C(ui, SURFACE_MUTED) : CUI_C(ui, SURFACE);
        bn->border = 1;
        cui_color border = cui__control_edge(ui, e->hover);
        cui_color focus_c = cui__edge_on(ui);
        border = cui_mix(border, focus_c, e->focus);
        if (has_error) border = CUI_C(ui, DANGER);
        bn->border_color = border;
        if (e->focus > 0.01f) {
            bn->elevation = CUI_ELEVATION_NONE;
        }
    }
    bn->opacity = disabled ? CUI__DISABLED : 1.0f;
    cui__mark_interactive(ui, box, CUI_IF_FOCUSABLE);
    cui__open(ui, box);
    const char *icon = o->icon ? o->icon : search_icon;
    if (o->prefix) {
        int pt = cui__text_node(ui, o->prefix, -1, CUI_FONT_BODY, size, CUI_C(ui, TEXT_MUTED), false);
        ui->nodes[pt].line_height = 1.3f;
        ui->nodes[pt].self_align = CUI_ALIGN_CENTER;
        ui->nodes[box].gap = 4;
    }
    if (icon) {
        int ib = cui__node_new(ui, 0);
        if (multiline) ui->nodes[ib].pad[0] = (line_h - 16) * 0.5f;
        cui__open(ui, ib);
        cui__icon_node(ui, icon, 16, CUI_C(ui, TEXT_MUTED));
        cui__close(ui);
    }
    int ar = cui__leaf(ui, area_id, CUI_FILL, multiline ? CUI_FILL : CUI_PX(line_h));
    cui_node *an_ = &ui->nodes[ar];
    an_->self_align = multiline ? CUI_ALIGN_STRETCH : CUI_ALIGN_CENTER;
    cui__edit_paint *p = cui__frame_alloc(ui, sizeof *p);
    p->buf = cui__frame_strdup(ui, buf, len);
    p->len = len;
    p->cursor = e->cursor; p->anchor = e->anchor;
    p->focused = ui->focus == id;
    p->password = password;
    p->multiline = multiline;
    p->scroll_x = e->scroll_x; p->scroll_y = e->scroll_y;
    p->caret_phase = ui->time - ui->caret_time;
    p->font = font; p->size = size;
    p->placeholder = o->placeholder;
    p->text = CUI_C(ui, FOREGROUND);
    p->placeholder_color = CUI_C(ui, TEXT_MUTED);
    p->caret = cui__emphasis(ui);
    p->selection = ui->accent == CUI_ACCENT_NEUTRAL ? cui_alpha(cui_oklch(0.6f, 0.13f, 250, 1), 0.28f) : cui_alpha(CUI_C(ui, ACCENT), 0.26f);
    p->line_h = line_h;
    if (multiline) {
        cui_rect a = area;
        p->line_count = cui__edit_lines(ui, font, size, p->buf, len, a.w > 0 ? a.w : 200, &p->lines);
    }
    an_->paint = cui__paint_edit;
    an_->paint_data = p;
    if (p->focused) cui__wakeup_after(ui, 0.53f);

    if (o->suffix) {
        int st_ = cui__text_node(ui, o->suffix, -1, CUI_FONT_BODY, size, CUI_C(ui, TEXT_MUTED), false);
        ui->nodes[st_].line_height = 1.3f;
        ui->nodes[st_].self_align = CUI_ALIGN_CENTER;
    }
    /* trailing affordances */
    if (o->clearable && len > 0 && !disabled && !o->read_only) {
        cui_button_opts bo;
        memset(&bo, 0, sizeof bo);
        bo.size = CUI_SIZE_SM;
        cui_push_id_int(ui, (int)id);
        if (cui_icon_button_ex(ui, CUI_ICON_X, &bo)) {
            cui__undo_push(ui, e, buf);   /* Ctrl+Z brings the text back */
            e->last_edit = 0;
            buf[0] = 0;
            e->cursor = e->anchor = 0;
            e->scroll_x = 0;
            flags |= CUI_CHANGED;
            ui->focus = id;
        }
        cui_pop_id(ui);
        cui_node *cb = &ui->nodes[ui->last_node];
        cb->w = CUI_PX(22); cb->h = CUI_PX(22);
        cb->ty = 0;
        ui->nodes[box].pad[1] = 8;
    }
    if (o->password) {
        cui_button_opts bo;
        memset(&bo, 0, sizeof bo);
        bo.size = CUI_SIZE_SM;
        bo.color = CUI_COLOR_TEXT_MUTED;   /* quiet like the field's leading icon */
        bo.tooltip = e->show_password ? "Hide password" : "Show password";
        cui_push_id_int(ui, (int)id);
        if (cui_icon_button_ex(ui, e->show_password ? CUI_ICON_EYE_OFF : CUI_ICON_EYE, &bo)) e->show_password = !e->show_password;
        cui_pop_id(ui);
        cui_node *cb = &ui->nodes[ui->last_node];
        cb->w = CUI_PX(26); cb->h = CUI_PX(26);
        ui->nodes[box].pad[1] = 6;
    }
    cui__close(ui);
    if (style == CUI__FIELD_BOXED && e->focus > 0.01f && ui->focus_visible) {
        float *fa = cui__frame_alloc(ui, sizeof(float));
        *fa = e->focus;
        ui->nodes[box].paint_data = fa;
        extern void cui__paint_focus_ring_node(cui_ctx *ui, cui_node *n, void *data);
        ui->nodes[box].paint = cui__paint_focus_ring_node;
        ui->nodes[box].flags |= CUI_NF_PAINT_AFTER;
    }
    if (outer >= 0) {
        cui__field_message(ui, o->hint, o->error);
        cui__close(ui);
    }
    ui->last_id = id;
    ui->text_unshaped = unshaped;
    return flags;
}

int cui_input_ex(cui_ctx *ui, const char *id, char *buf, size_t cap, const cui_input_opts *o) {
    return cui__text_field(ui, cui_make_id(ui, id), buf, cap, o, false, CUI__FIELD_BOXED, NULL);
}

int cui_textarea_ex(cui_ctx *ui, const char *id, char *buf, size_t cap, const cui_input_opts *o) {
    return cui__text_field(ui, cui_make_id(ui, id), buf, cap, o, true, CUI__FIELD_BOXED, NULL);
}

int cui_search_ex(cui_ctx *ui, const char *id, char *buf, size_t cap, const cui_input_opts *o) {
    cui_input_opts d;
    memset(&d, 0, sizeof d);
    if (o) d = *o;
    d.clearable = true;
    if (!d.placeholder) d.placeholder = "Search";
    return cui__text_field(ui, cui_make_id(ui, id), buf, cap, &d, false, CUI__FIELD_BOXED, CUI_ICON_SEARCH);
}

/* ---- number field ---- */

typedef struct { char text[64]; double last; bool init; bool editing; } cui__number_state;

bool cui_number_ex(cui_ctx *ui, const char *id_str, double *value, double min, double max, const cui_number_opts *o) {
    cui_number_opts d;
    memset(&d, 0, sizeof d);
    if (!o) o = &d;
    cui_id id = cui_make_id(ui, id_str);
    cui__number_state *st = cui_state(ui, id ^ 0x4E554Du, sizeof *st);
    double step = o->step > 0 ? o->step : 1;
    double old = *value;
    bool focused = ui->focus == id;
    if (!st->init || (!focused && st->last != *value)) {
        snprintf(st->text, sizeof st->text, "%.*f", o->decimals, *value);
        st->last = *value;
        st->init = true;
    }
    int col = -1;
    if (o->label) {
        col = cui__node_new(ui, 0);
        ui->nodes[col].gap = 6;
        cui__soft_width(&ui->nodes[col], o->width, 160);
        cui__open(ui, col);
        cui__field_label(ui, o->label);
    }
    int row = cui__node_new(ui, 0);
    cui_node *rn = &ui->nodes[row];
    rn->dir = CUI_ROW; rn->gap = 0; rn->align = CUI_ALIGN_CENTER;
    if (col >= 0) rn->w = CUI_FILL; else cui__soft_width(rn, o->width, 160);
    cui__open(ui, row);
    cui_input_opts io;
    memset(&io, 0, sizeof io);
    io.disabled = o->disabled;
    int f = cui__text_field(ui, id, st->text, sizeof st->text, &io, false, CUI__FIELD_BOXED, NULL);
    cui_node *box = &ui->nodes[ui->last_node];
    (void)box;
    /* find the field node (id) to add steppers inside its row */
    bool commit = (f & CUI_SUBMITTED) || (focused && ui->focus != id);
    if (f & CUI_CHANGED) {
        char *end = NULL;
        double v = strtod(st->text, &end);
        if (end != st->text) { *value = v; }
    }
    if (ui->focus == id) {
        if (cui_key_pressed(ui, CUI_KEY_UP)) *value += step * ((ui->mods & CUI_MOD_SHIFT) ? 10 : 1);
        if (cui_key_pressed(ui, CUI_KEY_DOWN)) *value -= step * ((ui->mods & CUI_MOD_SHIFT) ? 10 : 1);
    }
    cui__close(ui);
    /* steppers overlay inside the right side of the field */
    {
        cui_node *rown = &ui->nodes[row];
        int field = rown->first;
        if (field >= 0) {
            cui_node *fn = &ui->nodes[field];
            fn->pad[1] = 4;
            cui__open(ui, field);
            if (o->suffix) {
                int t = cui__text_node(ui, o->suffix, -1, CUI_FONT_BODY, cui__fs(ui, ui->theme.text_body), CUI_C(ui, TEXT_MUTED), false);
                ui->nodes[t].line_height = 1.3f;
            }
            cui_button_opts bo;
            memset(&bo, 0, sizeof bo);
            bo.size = CUI_SIZE_SM;
            bo.disabled = o->disabled || *value <= min;
            bo.tooltip = "Decrease";
            cui_push_id(ui, id_str);
            if (cui_icon_button_ex(ui, CUI_ICON_MINUS, &bo)) { *value -= step; }
            ui->nodes[ui->last_node].w = CUI_PX(26); ui->nodes[ui->last_node].h = CUI_PX(26);
            bo.disabled = o->disabled || *value >= max;
            bo.tooltip = "Increase";
            if (cui_icon_button_ex(ui, CUI_ICON_PLUS, &bo)) { *value += step; }
            ui->nodes[ui->last_node].w = CUI_PX(26); ui->nodes[ui->last_node].h = CUI_PX(26);
            cui_pop_id(ui);
            cui__close(ui);
        }
    }
    if (col >= 0) cui__close(ui);
    if (*value < min) *value = min;
    if (*value > max) *value = max;
    if (*value != old || commit) {
        if (ui->focus != id || *value != old) {
            if (!(f & CUI_CHANGED)) snprintf(st->text, sizeof st->text, "%.*f", o->decimals, *value);
        }
        st->last = *value;
    }
    ui->last_id = id;
    return *value != old;
}

/* ---- primitives shared with the code editor (cui_editor.c) ---- */

void cui__edit_attach(cui_ctx *ui, cui_id id, cui_edit_state *e) {
    if (e->init) return;
    e->init = true;
    e->preferred_x = -1;
    cui__state_set_dtor(ui, id, cui__edit_dtor);
}
void cui__edit_undo_push(cui_ctx *ui, cui_edit_state *e, const char *buf, int kind) { cui__undo_push_kind(ui, e, buf, kind); }
int cui__edit_insert(char *buf, int *len, size_t cap, int pos, const char *s, int n) { return cui__insert(buf, len, cap, pos, s, n, true); }
void cui__edit_delete(char *buf, int *len, int a, int b) { cui__delete_range(buf, len, a, b); }
bool cui__edit_is_word(unsigned char c) { return cui__is_word(c); }
int cui__edit_word_left(const char *s, int pos) { return cui__word_left(s, pos); }
int cui__edit_word_right(const char *s, int len, int pos) { return cui__word_right(s, len, pos); }
