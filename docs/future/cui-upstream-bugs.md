# CUI upstream bugs

## Right skips toggles in a toggle group

Reproduced with CUI v1.4.0 at `5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`
on Windows with the software renderer. The program uses CUI directly.

```c
#include <cui.h>
#include <stdio.h>

static void frame(cui_ctx *ui, bool *selected, cui_key key) {
    const char *labels[] = { "First", "Second", "Third", "Fourth" };
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, 1.0f / 60.0f);
    cui_toggle_group(ui, "focus-probe", labels, 4, selected);
    cui_end_frame(ui);
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    cui_set_viewport(ui, 400, 120, 1);
    bool selected[4] = { false };
    frame(ui, selected, CUI_KEY_NONE);
    frame(ui, selected, CUI_KEY_TAB);
    frame(ui, selected, CUI_KEY_RIGHT);
    frame(ui, selected, CUI_KEY_ENTER);
    printf("After Tab, Right, Enter: %d %d %d %d\n",
        selected[0], selected[1], selected[2], selected[3]);
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}
```

Tab focuses the first toggle. One Right should move focus to the second, so
Enter should yield `0 1 0 0`. The program printed the following output.

```text
After Tab, Right, Enter: 0 0 0 1
```

`cui_toggle_group_ex` changes focus inside its forward loop. Each subsequent
item sees the same key press after acquiring focus. Tab and Left can reach the
intermediate items. FPAS currently delegates navigation to CUI.
[CUI implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_actions.c)

- [ ] Submit an upstream PR with a keyboard regression test for a single Right
  press. Check the current upstream code and issue list before submitting.

## Range slider takes focus without keyboard adjustment

Reproduced on Windows with CUI v1.4.0 at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` using the software renderer and
direct CUI calls.

```c
#include <cui.h>
#include <stdio.h>

static void frame(cui_ctx *ui, float *lo, float *hi, float *value, cui_key key) {
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, 1.0f / 60.0f);
    cui_range_slider(ui, "Range", lo, hi, 0, 100);
    cui_slider(ui, "Slider", value, 0, 100);
    cui_end_frame(ui);
}

int main(void) {
    cui_sw_renderer *renderer = cui_sw_create();
    const cui_ctx_desc settings = { .renderer = cui_sw_iface(renderer) };
    cui_ctx *ui = cui_create(&settings);
    cui_set_viewport(ui, 400, 200, 1);
    float lo = 20, hi = 75, value = 40;
    frame(ui, &lo, &hi, &value, CUI_KEY_NONE);
    for (int tab = 1; tab <= 2; ++tab) {
        frame(ui, &lo, &hi, &value, CUI_KEY_TAB);
        frame(ui, &lo, &hi, &value, CUI_KEY_RIGHT);
        printf("Tab %d, Right: range %.0f..%.0f, slider %.0f\n", tab, lo, hi, value);
    }
    cui_destroy(ui);
    cui_sw_destroy(renderer);
}
```

The first Tab stops on the range track. Right leaves both endpoints unchanged.
The second Tab reaches the single-value slider, where Right changes the value.

```text
Tab 1, Right: range 20..75, slider 40
Tab 2, Right: range 20..75, slider 41
```

`cui_range_slider` registers a focusable track and only adjusts its endpoints
during a pointer drag. FPAS uses the native interaction.
[CUI implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_controls.c)

- [ ] Check current upstream behavior and decide on keyboard adjustment and a
  visible focus indicator before preparing a PR.

## Breadcrumb links and numbered page buttons have invisible focus

Reproduced with direct CUI calls on Windows using the software renderer in
CUI v1.4.0 at `5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`.

```c
#include <cui.h>
#include <stdio.h>
#include <string.h>

enum { WIDTH = 400, HEIGHT = 120, STRIDE = WIDTH * 4 };
static unsigned char initial[HEIGHT * STRIDE], focused[HEIGHT * STRIDE];

static int frame(cui_ctx *ui, int *page, cui_key key) {
    const char *path[] = { "Home", "Projects", "Settings" };
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, 1.0f / 60.0f);
    int ancestor = cui_breadcrumb(ui, path, 3);
    cui_pagination(ui, "pages", page, 3);
    cui_end_frame(ui);
    return ancestor;
}

static void render(cui_sw_renderer *sw, cui_ctx *ui, unsigned char *pixels) {
    cui_sw_render(sw, cui_get_draw_data(ui), pixels, WIDTH, HEIGHT, STRIDE, CUI_SW_RGBA);
}

int main(void) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_sw_set_incremental(sw, false);
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, WIDTH, HEIGHT, 1);
    cui_set_reduced_motion(ui, true);
    cui_input_mouse_leave(ui);

    int page = 0;
    frame(ui, &page, CUI_KEY_NONE);
    render(sw, ui, initial);
    frame(ui, &page, CUI_KEY_TAB);
    render(sw, ui, focused);
    int ancestor = frame(ui, &page, CUI_KEY_ENTER);
    printf("Breadcrumb focus changes pixels: %d, Enter returns: %d\n",
        memcmp(initial, focused, sizeof initial) != 0, ancestor);

    for (int step = 0; step < 3; ++step) frame(ui, &page, CUI_KEY_TAB);
    render(sw, ui, focused);
    frame(ui, &page, CUI_KEY_ENTER);
    printf("Page button focus changes pixels: %d, Enter selects: %d\n",
        memcmp(initial, focused, sizeof initial) != 0, page);

    cui_destroy(ui);
    cui_sw_destroy(sw);
}
```

Tab moves to Home and Enter activates it. Three more Tab presses reach page
two, and Enter selects it. In both cases the focused frame matches the
unfocused frame byte for byte. Keyboard users cannot see which breadcrumb
link or numbered page button they are about to activate.

```text
Breadcrumb focus changes pixels: 0, Enter returns: 0
Page button focus changes pixels: 0, Enter selects: 1
```

`cui_breadcrumb` and `cui__page_button` register focusable nodes. Neither
uses the interaction's focus visibility to draw a cue. FPAS uses the native
presentation. [CUI implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_controls.c)

- [ ] Check current upstream behavior and prepare a keyboard focus indicator
  with a rendering regression test.

## Pagination overflows an unnamed narrow parent

Reproduced with direct CUI calls on Windows using CUI v1.4.0 at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5`. The viewport is 360 points
wide, and the parent has 280 points of usable width after padding.

```c
#include <cui.h>
#include <stdio.h>

static float next_edge(const char *parent_id) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, 360, 160, 1);
    int page = 5;
    for (int frame = 0; frame < 3; ++frame) {
        cui_begin_frame(ui, 1.0f / 60.0f);
        cui_box_begin(ui, &(cui_box){ .id = parent_id, .width = CUI_PX(328), .pad = 24 });
        cui_pagination(ui, "pages", &page, 12);
        cui_box_end(ui);
        cui_end_frame(ui);
    }
    cui_rect next = cui_last_rect(ui);
    cui_destroy(ui);
    cui_sw_destroy(sw);
    return next.x + next.w;
}

int main(void) {
    printf("Inner right edge: 304, unnamed parent: %.0f, named parent: %.0f\n",
        next_edge(NULL), next_edge("card"));
}
```

```text
Inner right edge: 304, unnamed parent: 320, named parent: 252
```

The unnamed box makes `cui_pagination` estimate space from the viewport
and select its wider layout. With a named box, CUI uses the parent's
previous rectangle and selects the compact layout. FPAS gives unnamed
pager parents a stable native ID. [CUI implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_controls.c)

- [ ] Check whether upstream can use the available width for unnamed parents
  and add coverage for the narrow gallery card.

## Steppers with the same first caption share their measured width

Reproduced with direct CUI calls on Windows using CUI v1.4.0 at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` and the software renderer.
Draw both pairs for three frames in a 950 × 130 viewport.

```c
#include <cui.h>
#include <stdlib.h>

static void pair(cui_ctx *ui, bool scoped) {
    const char *steps[] = { "Account", "Workspace", "Invite team", "Done" };
    cui_box_begin(ui, &(cui_box){ .dir = CUI_ROW, .gap = 16, .pad = 8 });
    for (int column = 0; column < 2; ++column) {
        cui_box_begin(ui, &(cui_box){ .width = CUI_PX(column == 0 ? 280 : 620) });
        if (scoped) cui_push_id(ui, column == 0 ? "narrow" : "wide");
        cui_stepper(ui, steps, 4, 1);
        if (scoped) cui_pop_id(ui);
        cui_box_end(ui);
    }
    cui_box_end(ui);
}

int main(int argc, char **argv) {
    if (argc != 2) return 1;
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, 950, 130, 1);
    for (int frame = 0; frame < 3; ++frame) {
        cui_begin_frame(ui, 1.0f / 60.0f);
        pair(ui, false);
        pair(ui, true);
        cui_end_frame(ui);
    }
    const cui_draw_data *draw = cui_get_draw_data(ui);
    int width = draw->width_px, height = draw->height_px;
    void *pixels = malloc((size_t)width * height * 4);
    cui_sw_render(sw, draw, pixels, width, height, width * 4, CUI_SW_RGBA);
    bool saved = cui_write_png(argv[1], pixels, width, height, width * 4);
    free(pixels);
    cui_destroy(ui);
    cui_sw_destroy(sw);
    return saved ? 0 : 1;
}
```

In the upper pair, the 280-point stepper keeps all four captions, truncates
them, and overlaps the stepper to its right. In the lower pair, the narrow
stepper keeps only “Workspace” and both fit in their columns. `cui_stepper`
looks up its previous rectangle using the first caption. FPAS gives each
stepper its own native ID scope so repeated captions can adapt independently.
[CUI implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_controls.c)

- [ ] Check current upstream behavior and add a regression case with two
  steppers sharing a first caption.

## Toast timing can outlast the deadline or accept an expired action

Reproduced with direct CUI calls on Windows at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` (v1.4.0).

```c
#include <cui.h>
#include <stdio.h>

static void paint(cui_ctx *ui, float dt, cui_key key) {
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, dt);
    cui_end_frame(ui);
}

static void idle(void) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){0});
    cui_set_viewport(ui, 500, 300, 1);
    cui_set_reduced_motion(ui, true);
    cui_begin_frame(ui, 0);
    cui_toast(ui, "Idle notice", .duration = 1.2f);
    cui_end_frame(ui);
    float wake = cui_next_wakeup(ui);
    printf("First wake: %.2fs\n", wake);
    paint(ui, wake, CUI_KEY_NONE);
    printf("Additional wake after deadline: %.2fs\n", cui_next_wakeup(ui));
    cui_destroy(ui);
}

static void late_action(void) {
    cui_ctx *ui = cui_create(&(cui_ctx_desc){0});
    cui_set_viewport(ui, 500, 300, 1);
    cui_begin_frame(ui, 0);
    int handle = cui_toast(ui, "Message archived", .duration = 1.0f, .action = "Undo");
    cui_end_frame(ui);
    paint(ui, 0.01f, CUI_KEY_TAB);
    for (int i = 0; i < 49; ++i) paint(ui, 0.02f, CUI_KEY_NONE);
    paint(ui, 0.02f, CUI_KEY_ENTER);
    printf("Undo after elapsed duration: %d\n", cui_toast_action_clicked(ui, handle));
    cui_destroy(ui);
}

int main(void) {
    idle();
    late_action();
}
```

```text
First wake: 1.21s
Additional wake after deadline: 0.96s
Undo after elapsed duration: 1
```

The first case advances the next frame by the requested wake interval. CUI caps its
delta at 250 ms, so most of the elapsed time is missing from the toast's
age. FPAS requests earlier wakeups while a timed toast is running.

In the second case Tab focuses Undo, and Enter arrives after the duration
while the toast still has its exit animation. CUI still builds the button
and reports activation. FPAS filters action events after the native
deadline. [CUI toast implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_overlay.c)

- [ ] Verify current upstream timing, preserve hover pause and cover
  idle expiry and input during the exit animation in a later contribution.

## Tooltip delay varies with startup and idle frame timing

Reproduced with direct CUI calls on Windows at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` (v1.4.0).

```c
#include <cui.h>
#include <stdio.h>

static cui_rect paint(cui_ctx *ui, float dt, bool hint) {
    cui_begin_frame(ui, dt);
    cui_button(ui, "Anchor");
    if (hint) cui_tooltip(ui, "Tooltip");
    cui_end_frame(ui);
    return cui_last_rect(ui);
}

static void run(bool hint, bool idle) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, 500, 300, 1);
    cui_set_reduced_motion(ui, true);
    cui_input_mouse_leave(ui);
    if (idle) paint(ui, 0.25f, hint);
    cui_rect anchor = paint(ui, idle ? 0.25f : 0.01f, hint);
    cui_input_mouse_move(ui, anchor.x + anchor.w / 2, anchor.y + anchor.h / 2);
    paint(ui, 0.01f, hint);
    float wake = cui_next_wakeup(ui);
    printf("%s %s: vertices=%d, wake=%.2fs\n", idle ? "Idle" : "Startup",
           hint ? "with hint" : "without hint", cui_get_draw_data(ui)->vertex_count, wake);
    if (idle && wake > 0) {
        paint(ui, wake, hint);
        printf("After requested interval: vertices=%d, wake=%.2fs\n",
               cui_get_draw_data(ui)->vertex_count, cui_next_wakeup(ui));
    }
    cui_destroy(ui);
    cui_sw_destroy(sw);
}

int main(void) {
    run(false, false);
    run(true, false);
    run(true, true);
}
```

```text
Startup without hint: vertices=28, wake=-1.00s
Startup with hint: vertices=60, wake=-1.00s
Idle with hint: vertices=28, wake=0.56s
After requested interval: vertices=28, wake=0.31s
```

On startup the first hovered button draws its tooltip immediately. The
zero-initialized last-show time falls within the 400 ms warm interval.
Later, an idle loop that sleeps for the requested interval still has to wait
for the same tooltip because CUI advances the clock by at most 250 ms per
frame. The FPAS bridge initializes the last-show time before the warm interval
and requests shorter wake intervals while an interactive tooltip is pending.
[CUI tooltip implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_overlay.c)

- [ ] Verify both timing cases on current upstream and cover them with
  regressions in a later contribution.

## Enter can open and immediately confirm a confirmation popover

Reproduced with direct CUI calls on Windows at
`5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5` (v1.4.0).

```c
#include <cui.h>
#include <stdio.h>

static void frame(cui_ctx *ui, cui_key key) {
    if (key != CUI_KEY_NONE) {
        cui_input_key(ui, key, true, 0);
        cui_input_key(ui, key, false, 0);
    }
    cui_begin_frame(ui, 0.016f);
    bool triggered = cui_button(ui, "Delete branch");
    if (triggered) cui_popover_open(ui, "branch");
    bool confirmed = cui_confirm_popover(ui, "branch", .title = "Delete feature/grid?", .danger = true);
    cui_end_frame(ui);
    if (key == CUI_KEY_ENTER) printf("trigger=%d confirmed=%d open=%d\n",
        triggered, confirmed, cui_popover_is_open(ui, "branch"));
}

int main(void) {
    cui_sw_renderer *sw = cui_sw_create();
    cui_ctx *ui = cui_create(&(cui_ctx_desc){ .renderer = cui_sw_iface(sw) });
    cui_set_viewport(ui, 640, 480, 1);
    cui_set_reduced_motion(ui, true);
    frame(ui, CUI_KEY_NONE);
    frame(ui, CUI_KEY_TAB);
    frame(ui, CUI_KEY_ENTER);
    cui_destroy(ui);
    cui_sw_destroy(sw);
}
```

```text
trigger=1 confirmed=1 open=0
```

Tab focuses the anchor button. Enter activates the button and opens the
confirmation panel. The panel's Enter fallback sees the same key press and
answers the question before it appears. FPAS opens through an MVU update and
consumes any Enter press in the frame in which the native question first opens.
[CUI confirmation implementation](https://github.com/yappologistic/CUI/blob/5a9c12b433d4501bc2f5632ead3a1cc05e5b1ee5/src/cui_disclosure.c)

- [ ] Verify current upstream and add a regression for opening by keyboard.
