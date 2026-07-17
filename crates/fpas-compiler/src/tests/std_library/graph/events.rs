use fpas_std::{GraphEvent, mouse_action_index, mouse_button_index};

use super::support::{compile_run_with_graph_events, with_headless};

#[test]
fn std_graph_on_mouse_down_dispatches_to_configured_handler() {
    with_headless(|| {
        let out = compile_run_with_graph_events(
            "\
program T;
uses Std.Console, Std.Graph;

mutable var Clicks: integer := 0;

procedure OnPaint(App: Application);
begin
end;

procedure OnMouse(App: Application; E: Std.Graph.Event);
begin
  if E.mouse_action = MouseAction.Down then
  begin
    Clicks := Clicks + 1;
    Application.HostRequestQuit(App)
  end
end;

begin
  var App: Application := Application.Open(32, 24, 'Graph mouse');
  var Handlers: ApplicationHandlers := record
    OnPaint := OnPaint;
    OnMouse := Some(OnMouse);
  end;
  Application.Configure(App, Handlers);
  Application.Run(App);
  WriteLn(Clicks)
end.",
            &[GraphEvent::Mouse {
                action: mouse_action_index("Down"),
                button: mouse_button_index("Left"),
                x: 8,
                y: 8,
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            }],
        );

        assert_eq!(out.lines, vec!["1"]);
    });
}
