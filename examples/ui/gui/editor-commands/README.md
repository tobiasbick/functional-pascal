# Native editor commands and shortcuts

Build/run `EditorCommands.fpasproj` with the local FPAS SDK. An optional first
program argument supplies a PNG/JPEG path for CUI's account avatar. The example
uses native go-to, range selection, find, Unicode highlights, custom tour
captions/count formatting and an aggregate selection bar without item flags.
The main widget gallery retains its upstream composition and defaults.

Ctrl/Cmd+S emits an application Save message while editing; no file is written.
F1 opens the tour. Bindings reuse `ShortcutRecorders.Combination` and
`PrimaryModifier`, and may map child messages with `Binding.Map`. CUI widgets
run first. Popups/modals own their keys, including the frame that dismisses
them. `AllowWhileEditing` is an explicit opt-in for global commands such as
Save; avoid assigning it to native editing combinations. Repeat is off unless
requested and waits for a key release observed by the frame loop.

Editor selections, observations and marks use UTF-16 offsets at Unicode scalar
boundaries; line/column navigation uses CUI's one-based visual columns and tab
stops. CUI owns finding, scrolling, editing and painting. Requests target the
next normal view, expire for absent/disabled editors and keep only the last
request per editor. A null find query closes the bar and restores editor focus.

`CountFormat` uses CUI's integer format: `%d` and literal `%%` only. Tours accept
up to two slots (step, total); selection bars accept one. This preserves native
count animation. See the [GUI reference](../../../../docs/ui/gui/host.md).
