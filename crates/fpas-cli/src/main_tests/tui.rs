use super::*;
use fpas_std::{ConsoleEvent, ConsoleKeyEvent, key_kind_index};
use std::sync::OnceLock;

#[test]
fn terminal_renderer_skips_unchanged_frames_and_flushes_damage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let cwd = create_temp_dir("tui-terminal-damage");
    let library = cwd.join("library");
    let source_glob = root
        .join("lib/Std/**/*.fpas")
        .to_string_lossy()
        .replace('\\', "/");
    write_text(
        &library.join("stdlib.fpasprj"),
        &format!(
            r#"[project]
name = "tui-terminal-renderer-test"
kind = "library"

[exports]
units = ["Std.Tui", "Std.Version", "Std.Tui.Runtime.TerminalRenderer"]

[sources]
include = ["{source_glob}"]
"#
        ),
    );

    let once = cwd.join("once.fpas");
    write_text(
        &once,
        r#"program FlushOnce;

uses Std.Tui as Tui; uses Std.Tui.Runtime.TerminalRenderer as TerminalRenderer;

begin
  var Surface: Tui.TuiWorkingSurface := Tui.TuiWorkingSurface.Create(Tui.TuiSize.Create(4, 2));
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
end program;
"#,
    );
    let unchanged = cwd.join("unchanged.fpas");
    write_text(
        &unchanged,
        r#"program FlushUnchanged;

uses Std.Tui as Tui; uses Std.Tui.Runtime.TerminalRenderer as TerminalRenderer;

begin
  var Surface: Tui.TuiWorkingSurface := Tui.TuiWorkingSurface.Create(Tui.TuiSize.Create(4, 2));
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
end program;
"#,
    );
    let changed = cwd.join("changed.fpas");
    write_text(
        &changed,
        r#"program FlushChanged;

uses Std.Tui as Tui; uses Std.Tui.Runtime.TerminalRenderer as TerminalRenderer;

begin
  var Surface: Tui.TuiWorkingSurface := Tui.TuiWorkingSurface.Create(Tui.TuiSize.Create(4, 2));
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
  Surface.PutGlyph(1, 0, 'X');
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
end program;
"#,
    );
    let wide = cwd.join("wide.fpas");
    write_text(
        &wide,
        r#"program FlushWideTransition;


uses Std.Console as Console; uses Std.Options as Options; uses Std.Test as Test; uses Std.Tui as Tui; uses Std.Tui.Runtime.TerminalRenderer as TerminalRenderer;

begin
  var Surface: Tui.TuiWorkingSurface := Tui.TuiWorkingSurface.Create(Tui.TuiSize.Create(4, 1));
  Surface.PutCell(Tui.TuiPoint.Create(1, 0), Tui.TuiCell.Create('中', Tui.TuiStyleRole.Accent));
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
  Surface.PutGlyph(2, 0, 'X');
  TerminalRenderer.TuiFlushSurface(Surface, Tui.TuiPalette.Default());
  Test.AssertEquals(' ', Options.Unwrap(Console.GetCell(2, 1)).glyph);
  Test.AssertEquals('X', Options.Unwrap(Console.GetCell(3, 1)).glyph);
end program;
"#,
    );

    let standard_library = fpas_project::load_standard_library(&library)
        .expect("terminal renderer standard library must load");
    let program_graph = fpas_project::prepare_program_unit_graph(
        &[],
        &fpas_project::ProjectLinkMeta::default(),
        Some(&standard_library),
    )
    .expect("terminal renderer program graph must build");
    let run = |program: &Path| {
        support::run_program_with_graph_and_capture_output(program, &program_graph)
    };
    let (once_exit, once_stdout, once_stderr) = run(&once);
    let (unchanged_exit, unchanged_stdout, unchanged_stderr) = run(&unchanged);
    let (changed_exit, changed_stdout, changed_stderr) = run(&changed);
    let (wide_exit, _wide_stdout, wide_stderr) = run(&wide);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(once_exit, 0, "stderr: {once_stderr}");
    assert_eq!(unchanged_exit, 0, "stderr: {unchanged_stderr}");
    assert_eq!(changed_exit, 0, "stderr: {changed_stderr}");
    assert_eq!(wide_exit, 0, "stderr: {wide_stderr}");
    assert_eq!(
        unchanged_stdout, once_stdout,
        "an unchanged second frame must not write terminal output"
    );
    assert!(
        changed_stdout.starts_with(&once_stdout),
        "the changed run must preserve the initial frame output"
    );
    assert!(
        changed_stdout[once_stdout.len()..].contains('X'),
        "the changed run must write the damaged cell"
    );
}

#[test]
fn theme_switch_repaints_unchanged_terminal_cells() {
    let cwd = create_temp_dir("tui-theme-switch");
    let program = cwd.join("theme_switch.fpas");
    write_text(
        &program,
        r#"program ThemeSwitch;

uses Std.Console as Console;
uses Std.Options as Options;
uses Std.Test as Test;
uses Std.Tui as Tui;

function UpdateTheme(State: integer; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): integer;
begin
  case Msg of
    when Tui.TuiMsg.Resize(const Size):
      begin
        Cmd.SetPalette(Tui.TuiPalette.Default()
                         .WithRole(Tui.TuiStyleRole.Normal, Tui.TuiStyle.FromColors(Tui.TuiColor.FromRgb(1, 2, 3), Tui.TuiColor.FromRgb(4, 5, 6))));
        return State + 1;
      end;
    when Tui.TuiMsg.QuitRequested:
      begin
        Cmd.Set(Tui.TuiCmd.Quit);
        return State;
      end;
    when Tui.TuiMsg.Started, Tui.TuiMsg.Key(_), Tui.TuiMsg.Pointer(_), Tui.TuiMsg.Tick(_), Tui.TuiMsg.FocusChanged(_, _), Tui.TuiMsg.Action(_, _), Tui.TuiMsg.TextChanged(_, _, _, _), Tui.TuiMsg.TextAreaChanged(_, _, _, _, _), Tui.TuiMsg.CheckChanged(_, _, _), Tui.TuiMsg.SelectionChanged(_, _, _), Tui.TuiMsg.ScrollChanged(_, _, _), Tui.TuiMsg.MenuChanged(_, _, _), Tui.TuiMsg.BackgroundFailed(_, _, _, _, _, _):
      begin
        begin
          return State;
        end;
      end;
  end case;
end function;

function ViewTheme(State: integer): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel('theme');
end function;

begin
  var Initial: Tui.TuiPalette := Tui.TuiPalette.Default()
                                   .WithRole(Tui.TuiStyleRole.Normal, Tui.TuiStyle.FromColors(Tui.TuiColor.FromRgb(10, 20, 30), Tui.TuiColor.FromRgb(40, 50, 60)));
  Test.AssertEquals(1, Tui.TuiApplication.RunWithPalette(0, UpdateTheme, ViewTheme, Initial));
  var Painted: Console.Cell := Options.Unwrap(Console.GetCell(1, 1));
  Test.AssertTrue(Painted.foreground.kind = Console.ColorKind.Rgb);
  Test.AssertEquals(1, Painted.foreground.red);
  Test.AssertEquals(2, Painted.foreground.green);
  Test.AssertEquals(3, Painted.foreground.blue);
  Test.AssertEquals(4, Painted.background.red);
  Test.AssertEquals(5, Painted.background.green);
  Test.AssertEquals(6, Painted.background.blue);
end program;
"#,
    );
    let built =
        crate::project_build::build_test_program_with_graph(&program, repo_tui_program_graph())
            .expect("Tui theme-switch regression program must build");
    let mut vm = fpas_vm::Vm::new(built.executable);
    vm.push_console_event(ConsoleEvent::resize(10, 2));
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\0',
        false,
        false,
        false,
        false,
    )));

    vm.run()
        .expect("Tui theme-switch regression program must run");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn interactive_host_debounces_resize_burst_after_quiet_period() {
    let cwd = create_temp_dir("tui-resize-debounce");
    let program = cwd.join("resize_debounce.fpas");
    write_text(
        &program,
        r#"program ResizeDebounce;

uses Std.Console as Console;
uses Std.Tui as Tui;

type Model = record
  ResizeCount: integer;
  Width: integer;
  Height: integer;
end record;

function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;
begin
  case Msg of
    when Tui.TuiMsg.Resize(const Size):
      begin
        Cmd.Set(Tui.TuiCmd.Quit);
        return Model(ResizeCount := State.ResizeCount + 1, Width := Size.Width, Height := Size.Height);
      end;
    when Tui.TuiMsg.Started, Tui.TuiMsg.Key(_), Tui.TuiMsg.Pointer(_), Tui.TuiMsg.Tick(_), Tui.TuiMsg.FocusChanged(_, _), Tui.TuiMsg.Action(_, _), Tui.TuiMsg.TextChanged(_, _, _, _), Tui.TuiMsg.TextAreaChanged(_, _, _, _, _), Tui.TuiMsg.CheckChanged(_, _, _), Tui.TuiMsg.SelectionChanged(_, _, _), Tui.TuiMsg.ScrollChanged(_, _, _), Tui.TuiMsg.MenuChanged(_, _, _), Tui.TuiMsg.BackgroundFailed(_, _, _, _, _, _), Tui.TuiMsg.QuitRequested:
      begin
        begin
          return State;
        end;
      end;
  end case;
end function;

function View(State: Model): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel('resize');
end function;

begin
  var Final: Model := Tui.TuiApplication.Run(Model(ResizeCount := 0, Width := 0, Height := 0), Update, View);
  Console.WriteLn(Final.ResizeCount);
  Console.WriteLn(Final.Width);
  Console.WriteLn(Final.Height);
end program;
"#,
    );
    let built =
        crate::project_build::build_test_program_with_graph(&program, repo_tui_program_graph())
            .expect("Tui resize-debounce regression program must build");
    let mut vm = fpas_vm::Vm::new(built.executable);
    vm.push_console_event(ConsoleEvent::resize(80, 24));
    vm.push_console_event(ConsoleEvent::resize(100, 30));
    vm.push_console_event(ConsoleEvent::resize(120, 40));

    vm.run()
        .expect("Tui resize-debounce regression program must run");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(vm.output().lines, vec!["1", "120", "40"]);
}

#[test]
fn interactive_host_does_not_emit_ticks_without_explicit_timer_input() {
    let cwd = create_temp_dir("tui-event-driven-idle");
    let program = cwd.join("event_driven_idle.fpas");
    write_text(
        &program,
        r#"program EventDrivenIdle;

uses Std.Console as Console;
uses Std.Tui as Tui;

type Model = record
  Ticks: integer;
end record;

function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;
begin
  case Msg of
    when Tui.TuiMsg.Tick(const Delta):
      begin
        return Model(Ticks := State.Ticks + 1);
      end;
    when Tui.TuiMsg.QuitRequested:
      begin
        Cmd.Set(Tui.TuiCmd.Quit);
        return State;
      end;
    when Tui.TuiMsg.Started, Tui.TuiMsg.Key(_), Tui.TuiMsg.Pointer(_), Tui.TuiMsg.Resize(_), Tui.TuiMsg.FocusChanged(_, _), Tui.TuiMsg.Action(_, _), Tui.TuiMsg.TextChanged(_, _, _, _), Tui.TuiMsg.TextAreaChanged(_, _, _, _, _), Tui.TuiMsg.CheckChanged(_, _, _), Tui.TuiMsg.SelectionChanged(_, _, _), Tui.TuiMsg.ScrollChanged(_, _, _), Tui.TuiMsg.MenuChanged(_, _, _), Tui.TuiMsg.BackgroundFailed(_, _, _, _, _, _):
      begin
        begin
          return State;
        end;
      end;
  end case;
end function;

function View(State: Model): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel('idle');
end function;

begin
  var Final: Model := Tui.TuiApplication.Run(Model(Ticks := 0), Update, View);
  Console.WriteLn(Final.Ticks);
end program;
"#,
    );
    let built =
        crate::project_build::build_test_program_with_graph(&program, repo_tui_program_graph())
            .expect("Tui idle regression program must build");
    let mut vm = fpas_vm::Vm::new(built.executable);
    vm.push_console_event(ConsoleEvent::focus_gained());
    vm.push_console_event(ConsoleEvent::key(ConsoleKeyEvent::new(
        key_kind_index("Escape"),
        '\0',
        false,
        false,
        false,
        false,
    )));

    vm.run().expect("Tui idle regression program must run");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        vm.output().lines,
        vec!["0"],
        "unsupported host events must not synthesize animation ticks"
    );
}

#[test]
fn interactive_background_message_wakes_idle_host_and_repaints() {
    let cwd = create_temp_dir("tui-background-wakeup");
    let program = cwd.join("background_wakeup.fpas");
    write_text(
        &program,
        r#"program BackgroundWakeup;

uses Std.Console as Console;
uses Std.Results as Results;
uses Std.Tasks as Tasks;
uses Std.Tui as Tui;

type Model = record
  Inbox: channel of (integer);
  Value: integer;
end record;

function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;
begin
  case Msg of
    when Tui.TuiMsg.Started:
      begin
        var Target: channel of (integer) := State.Inbox;
        Cmd.StartBackground(1, function(Token: Tasks.CancellationToken): Result of (boolean, string) begin
          return Tasks.SendWithCancellation(Target, 42, Token);
        end function);
        return State;
      end;
    when Tui.TuiMsg.BackgroundFailed(const Id, const Kind, const Message, const Code, const Line, const Column):
      begin
        panic(Message);
      end;
    when Tui.TuiMsg.Key(_), Tui.TuiMsg.Pointer(_), Tui.TuiMsg.Tick(_), Tui.TuiMsg.Resize(_), Tui.TuiMsg.FocusChanged(_, _), Tui.TuiMsg.Action(_, _), Tui.TuiMsg.TextChanged(_, _, _, _), Tui.TuiMsg.TextAreaChanged(_, _, _, _, _), Tui.TuiMsg.CheckChanged(_, _, _), Tui.TuiMsg.SelectionChanged(_, _, _), Tui.TuiMsg.ScrollChanged(_, _, _), Tui.TuiMsg.MenuChanged(_, _, _), Tui.TuiMsg.QuitRequested:
      begin
        begin
          return State;
        end;
      end;
  end case;
end function;

function UpdateApplication(State: Model; Message: integer; Cmd: Tui.TuiCmdOutput): Model;
begin
  Cmd.Set(Tui.TuiCmd.Quit);
  return Model(Inbox := State.Inbox, Value := Message);
end function;

function View(State: Model): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel('value');
end function;

begin
  var Inbox: channel of (integer) := Tasks.CreateChannel(1);
  var Final: Model := Tui.TuiApplication.RunWithBackground(Model(Inbox := Inbox, Value := 0), Inbox, Update, UpdateApplication, View);
  Console.WriteLn(Final.Value);
end program;
"#,
    );
    let built =
        crate::project_build::build_test_program_with_graph(&program, repo_tui_program_graph())
            .expect("Tui background wakeup regression program must build");
    let mut vm = fpas_vm::Vm::new(built.executable);

    vm.run()
        .expect("Tui background wakeup regression program must run");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(vm.output().lines, vec!["42"]);
}

fn run_repo_std_program(rel_path: &str) -> (i32, String, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let program = root.join(rel_path);
    support::run_program_with_graph_and_capture_output(&program, repo_tui_program_graph())
}

fn repo_tui_program_graph() -> &'static fpas_project::ProgramUnitGraph {
    static GRAPH: OnceLock<fpas_project::ProgramUnitGraph> = OnceLock::new();
    GRAPH.get_or_init(|| {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root");
        let standard_library = fpas_project::load_standard_library(&root.join("lib"))
            .expect("standard library must load");
        fpas_project::prepare_program_unit_graph(
            &[],
            &fpas_project::ProjectLinkMeta::default(),
            Some(&standard_library),
        )
        .expect("Tui test program graph must build")
    })
}

#[test]
fn rejects_duplicate_control_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/duplicate_control_id_runtime_error.fpas");

    assert_ne!(exit, 0, "duplicate Tui control ids must fail");
    assert!(
        stderr.contains("Tui control id must be unique in one tree: 1"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_forged_non_positive_element_control_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_element_control_id_runtime_error.fpas");

    assert_ne!(exit, 0, "non-positive Tui element control ids must fail");
    assert!(
        stderr.contains("Tui interactive elements require a positive control id"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_forged_non_positive_element_action_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_element_action_id_runtime_error.fpas");

    assert_ne!(exit, 0, "non-positive Tui element action ids must fail");
    assert!(
        stderr.contains("Tui interactive elements require a positive action id"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_invalid_cell_glyphs() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_cell_glyph_runtime_error.fpas");

    assert_ne!(exit, 0, "empty Tui cell glyphs must fail");
    assert!(
        stderr.contains("GraphemeWidth requires one non-zero-width extended grapheme cluster"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_rgb_channels_outside_the_byte_range_by_name() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_rgb_channel_runtime_error.fpas");

    assert_ne!(exit, 0, "RGB channels above 255 must fail");
    assert!(
        stderr.contains("Tui green channel is outside its supported range"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_cell_grid_length_mismatch() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_cell_grid_runtime_error.fpas");

    assert_ne!(exit, 0, "mismatched Tui cell-grid lengths must fail");
    assert!(
        stderr.contains("Tui cell grid length must equal width times height"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_gauge_values_above_the_maximum() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_gauge_runtime_error.fpas");

    assert_ne!(exit, 0, "out-of-range Tui gauge values must fail");
    assert!(
        stderr.contains("Tui gauge value must be between zero and its maximum"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_text_area_caret_outside_text() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/text_area_invalid_caret_runtime_error.fpas");

    assert_ne!(exit, 0, "out-of-range Tui text-area caret must fail");
    assert!(
        stderr.contains("Tui text area caret must be within its text"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_negative_text_area_offset() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/text_area_invalid_offset_runtime_error.fpas");

    assert_ne!(exit, 0, "negative Tui text-area offset must fail");
    assert!(
        stderr.contains("Tui text area offset must not be negative"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_negative_fixed_layout_height() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/fixed_height_negative_runtime_error.fpas");

    assert_ne!(exit, 0, "negative Tui fixed layout height must fail");
    assert!(
        stderr.contains("Tui fixed layout height must not be negative"),
        "stderr: {stderr}"
    );
}

#[test]
fn rejects_negative_fixed_layout_width() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/fixed_width_negative_runtime_error.fpas");

    assert_ne!(exit, 0, "negative Tui fixed layout width must fail");
    assert!(
        stderr.contains("Tui fixed layout width must not be negative"),
        "stderr: {stderr}"
    );
}

mod row;
