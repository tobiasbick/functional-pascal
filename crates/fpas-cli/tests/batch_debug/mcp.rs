use super::*;

#[test]
fn mcp_registration_does_not_grant_foreign_tasks_mutable_capture_access() {
    let fixture = Fixture::new(
        r#"program ForeignMcp;
uses Std.Ai.Mcp as Mcp, Std.Tasks as Tasks;
function Foreign(Server: Mcp.Server): boolean;
begin
  return Mcp.ServeOne(Server, Some(0), None).Unwrap();
end function;
begin
  var Calls := 0;
  const Handler: Mcp.ToolHandler := function(Context: Mcp.RequestContext; Arguments: Mcp.RawJson): result of (Mcp.ToolResult, Mcp.McpError)
  begin
    Calls := Calls + 1;
    return Ok(Mcp.TextResult('unexpected'));
  end function;
  const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap();
  discard Mcp.RegisterTool(Server, Mcp.ToolDefinition(Name := 'owned', Description := 'Local state', InputSchema := Mcp.RawJson(Text := '{"type":"object"}'), OutputSchema := None, Annotations := None, Meta := None), Handler).Unwrap();
  scope
    const Child := go Foreign(Server);
    discard Tasks.Wait(Child);
  end scope;
end.
"#,
    );
    let report = fixture.run(&["debug", "main.fpas", "--timeout", "10"], 2);
    assert_eq!(report["outcome"], "runtime_error");
    assert!(report["diagnostics"].as_array().unwrap().iter().any(|d| {
        d["message"]
            .as_str()
            .is_some_and(|text| text.contains("foreign task"))
    }));
}

#[test]
fn mcp_handler_survives_gc_and_suspends_in_the_debugger_with_an_independent_peer() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new(env!("CARGO_BIN_EXE_fpas"))
        .current_dir(root)
        .args([
            "debug",
            "tests/fixtures/mcp_debug_server.fpas",
            "--timeout",
            "30",
            "--instruction-limit",
            "1000000",
            "--gc-cell-threshold",
            "1",
            "--gc-mutation-threshold",
            "1",
            "--gc-poll-interval",
            "1",
        ])
        .output()
        .unwrap();
    // Reports can contain retired HTTP credentials in captured locals; do not print them.
    assert!(output.status.success(), "MCP debugger fixture failed");
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["outcome"], "completed");
    assert_eq!(value["exit_code"], 0);
}
